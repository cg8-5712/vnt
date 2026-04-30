package io.github.cg85712.vnt2.vpn

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import android.util.Log
import androidx.core.app.NotificationCompat
import com.vnt.RegisterResult
import com.vnt.VntManager
import com.vnt.VntNetwork
import io.github.cg85712.vnt2.MainActivity
import io.github.cg85712.vnt2.R

class VntVpnService : VpnService() {
  private val sessionLock = Any()
  private var sessionGeneration: Int = 0
  private var network: VntNetwork? = null
  private var vpnInterface: ParcelFileDescriptor? = null

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    val action = intent?.action ?: ACTION_START
    Log.i(TAG, "onStartCommand action=$action startId=$startId")
    return when (action) {
      ACTION_STOP -> {
        stopSession("连接已停止", clearPersisted = true, stopService = true)
        START_NOT_STICKY
      }

      ACTION_START, ACTION_RESTART -> {
        val request = readLaunchRequest(intent) ?: VntVpnRuntime.loadPersistedRequest(this)
        if (request == null) {
          VntVpnRuntime.markStartFailed("启动失败: 缺少配置")
          stopForegroundCompat()
          stopSelf()
          return START_NOT_STICKY
        }

        val config = VntVpnRuntime.parseLaunchConfig(request)
        val generation = nextGeneration()
        VntVpnRuntime.persistRequest(this, request)
        VntVpnRuntime.beginStart(request, config, "正在启动 ${config.configName}")
        startForegroundCompat("VNT 正在连接", config.configName)
        startSessionAsync(request, config, generation)
        START_STICKY
      }

      else -> START_NOT_STICKY
    }
  }

  override fun onRevoke() {
    stopSession("VPN 权限已被系统撤销", clearPersisted = true, stopService = true)
    super.onRevoke()
  }

  override fun onDestroy() {
    stopSession(message = null, clearPersisted = false, stopService = false)
    super.onDestroy()
  }

  private fun startSessionAsync(
    request: VntLaunchRequest,
    config: VntLaunchConfig,
    generation: Int,
  ) {
    Thread {
      var localNetwork: VntNetwork? = null
      var localVpnInterface: ParcelFileDescriptor? = null
      var nativeTunFd: Int? = null
      try {
        releaseHandles()
        ensureActive(generation)

        Log.i(TAG, "initializing VNT runtime")
        VntVpnRuntime.appendLog("正在初始化 VNT")
        if (!VntManager.init()) {
          throw IllegalStateException("无法初始化 VNT")
        }
        ensureActive(generation)

        localNetwork = VntManager.createNetwork(request.configJson)
          ?: throw IllegalStateException("无法创建网络实例")
        ensureActive(generation)

        Log.i(TAG, "registering VNT network for ${config.configName}")
        VntVpnRuntime.appendLog("正在注册网络")
        val registerResult = localNetwork.register()
        ensureActive(generation)
        Log.i(TAG, "registered overlay address ${registerResult.ip}/${registerResult.prefixLen}")
        VntVpnRuntime.appendLog("已获取地址 ${registerResult.ip}/${registerResult.prefixLen}")

        localVpnInterface = establishVpn(config, registerResult)
        ensureActive(generation)
        Log.i(TAG, "vpn interface established")
        VntVpnRuntime.appendLog("VPN 接口已建立")

        val duplicatedTunFd = duplicateTunFdForNative(localVpnInterface)
        nativeTunFd = duplicatedTunFd
        Log.i(TAG, "starting native TUN pipeline with duplicated fd")
        VntVpnRuntime.appendLog("正在启动 TUN 转发")
        localNetwork.startTun(duplicatedTunFd)
        nativeTunFd = null
        ensureActive(generation)
        val api = localNetwork.getApi()

        synchronized(sessionLock) {
          network = localNetwork
          vpnInterface = localVpnInterface
        }

        Log.i(TAG, "vpn session is running")
        VntVpnRuntime.markRunning(localNetwork, api)
        updateNotification("VNT 已连接", "${registerResult.ip}/${registerResult.prefixLen}")
      } catch (exception: Exception) {
        Log.e(TAG, "failed to start VPN session", exception)
        nativeTunFd?.let { fd ->
          closeDetachedFd(fd)
          nativeTunFd = null
        }
        try {
          localNetwork?.stop()
        } catch (stopException: Exception) {
          Log.w(TAG, "failed to stop local network after start failure", stopException)
        }
        try {
          localVpnInterface?.close()
        } catch (closeException: Exception) {
          Log.w(TAG, "failed to close vpn interface after start failure", closeException)
        }
        if (isActive(generation)) {
          VntManager.destroy()
          VntVpnRuntime.markStartFailed("启动失败: ${exception.message ?: exception.javaClass.simpleName}")
          VntVpnRuntime.clearPersistedRequest(this)
          stopForegroundCompat()
          stopSelf()
        }
      }
    }.start()
  }

  private fun establishVpn(
    config: VntLaunchConfig,
    registerResult: RegisterResult,
  ): ParcelFileDescriptor {
    val underlyingNetworks = resolveUnderlyingNetworks()
    val builder = Builder()
      .setSession("VNT2")
      .setMtu(config.mtu)
      .setConfigureIntent(createOpenAppPendingIntent())
      .addAddress(registerResult.ip, registerResult.prefixLen)

    try {
      builder.addDisallowedApplication(packageName)
      Log.i(TAG, "excluded VPN app process from VPN routing: $packageName")
    } catch (exception: Exception) {
      Log.w(TAG, "failed to exclude app from VPN routing", exception)
    }

    if (underlyingNetworks.isNotEmpty()) {
      builder.setUnderlyingNetworks(underlyingNetworks)
      Log.i(TAG, "attached ${underlyingNetworks.size} underlying network(s) to VPN session")
    } else {
      Log.w(TAG, "no underlying networks resolved for VPN session")
    }

    val overlayRoute = "${networkAddress(registerResult.ip, registerResult.prefixLen)}/${registerResult.prefixLen}"
    val routeSet = linkedSetOf(overlayRoute)
    for (route in config.outputRoutes) {
      val parts = route.split("/", limit = 2)
      if (parts.size != 2) {
        continue
      }

      val prefixLen = parts[1].toIntOrNull() ?: continue
      routeSet.add("${networkAddress(parts[0], prefixLen)}/$prefixLen")
    }

    for (route in routeSet) {
      val parts = route.split("/", limit = 2)
      if (parts.size != 2) {
        continue
      }

      val prefixLen = parts[1].toIntOrNull() ?: continue
      builder.addRoute(networkAddress(parts[0], prefixLen), prefixLen)
    }

    return builder.establish() ?: throw IllegalStateException("无法建立 VPN 接口")
  }

  private fun resolveUnderlyingNetworks(): Array<Network> {
    return try {
      val connectivityManager =
        getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager ?: return emptyArray()
      val networks = connectivityManager.allNetworks.filter { network ->
        val capabilities = connectivityManager.getNetworkCapabilities(network) ?: return@filter false
        capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) &&
          !capabilities.hasTransport(NetworkCapabilities.TRANSPORT_VPN)
      }
      networks.toTypedArray()
    } catch (exception: SecurityException) {
      Log.w(TAG, "ACCESS_NETWORK_STATE is unavailable; skipping underlying network binding", exception)
      emptyArray()
    }
  }

  private fun createOpenAppPendingIntent(): PendingIntent {
    val intent = packageManager.getLaunchIntentForPackage(packageName)
      ?: Intent(this, MainActivity::class.java)
    intent.addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP)
    return PendingIntent.getActivity(
      this,
      1,
      intent,
      PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
    )
  }

  private fun createStopPendingIntent(): PendingIntent {
    val intent = Intent(this, VntVpnService::class.java).setAction(ACTION_STOP)
    return PendingIntent.getService(
      this,
      2,
      intent,
      PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
    )
  }

  private fun stopSession(message: String?, clearPersisted: Boolean, stopService: Boolean) {
    Log.i(
      TAG,
      "stopSession clearPersisted=$clearPersisted stopService=$stopService message=${message ?: "<none>"}",
    )
    invalidateSessions()
    releaseHandles()
    VntManager.destroy()
    VntVpnRuntime.markStopped(message)
    if (clearPersisted) {
      VntVpnRuntime.clearPersistedRequest(this)
    }
    if (stopService) {
      stopForegroundCompat()
      stopSelf()
    }
  }

  private fun releaseHandles() {
    synchronized(sessionLock) {
      Log.i(
        TAG,
        "releaseHandles networkPresent=${network != null} vpnInterfacePresent=${vpnInterface != null}",
      )
      try {
        network?.stop()
      } catch (stopException: Exception) {
        Log.w(TAG, "failed to stop native network", stopException)
      }
      network = null

      try {
        vpnInterface?.close()
      } catch (closeException: Exception) {
        Log.w(TAG, "failed to close vpn interface", closeException)
      }
      vpnInterface = null
    }
  }

  private fun duplicateTunFdForNative(vpnInterface: ParcelFileDescriptor): Int {
    val duplicate = ParcelFileDescriptor.dup(vpnInterface.fileDescriptor)
    return duplicate.detachFd()
  }

  private fun closeDetachedFd(fd: Int) {
    try {
      ParcelFileDescriptor.adoptFd(fd).close()
    } catch (closeException: Exception) {
      Log.w(TAG, "failed to close detached tun fd=$fd", closeException)
    }
  }

  private fun startForegroundCompat(title: String, text: String) {
    ensureNotificationChannel()
    startForeground(NOTIFICATION_ID, buildNotification(title, text))
  }

  private fun updateNotification(title: String, text: String) {
    ensureNotificationChannel()
    val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    manager.notify(NOTIFICATION_ID, buildNotification(title, text))
  }

  private fun buildNotification(title: String, text: String) =
    NotificationCompat.Builder(this, NOTIFICATION_CHANNEL_ID)
      .setSmallIcon(R.mipmap.ic_launcher)
      .setContentTitle(title)
      .setContentText(text)
      .setContentIntent(createOpenAppPendingIntent())
      .setOngoing(true)
      .setOnlyAlertOnce(true)
      .addAction(0, "停止", createStopPendingIntent())
      .build()

  private fun ensureNotificationChannel() {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
      return
    }

    val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    if (manager.getNotificationChannel(NOTIFICATION_CHANNEL_ID) != null) {
      return
    }

    val channel = NotificationChannel(
      NOTIFICATION_CHANNEL_ID,
      "VNT VPN",
      NotificationManager.IMPORTANCE_LOW,
    )
    channel.description = "VNT VPN connection status"
    manager.createNotificationChannel(channel)
  }

  private fun stopForegroundCompat() {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
      stopForeground(STOP_FOREGROUND_REMOVE)
    } else {
      @Suppress("DEPRECATION")
      stopForeground(true)
    }
  }

  private fun readLaunchRequest(intent: Intent?): VntLaunchRequest? {
    if (intent == null) {
      return null
    }

    val fileName = intent.getStringExtra(EXTRA_FILE_NAME)
    val configName = intent.getStringExtra(EXTRA_CONFIG_NAME)
    val configJson = intent.getStringExtra(EXTRA_CONFIG_JSON)
    if (fileName.isNullOrBlank() || configName.isNullOrBlank() || configJson.isNullOrBlank()) {
      return null
    }
    return VntLaunchRequest(fileName, configName, configJson)
  }

  private fun networkAddress(ip: String, prefixLen: Int): String {
    if (prefixLen <= 0) {
      return "0.0.0.0"
    }
    if (prefixLen >= 32) {
      return ip
    }

    val parts = ip.split('.')
    if (parts.size != 4) {
      return ip
    }

    var value = 0L
    for (part in parts) {
      value = (value shl 8) or (part.toLongOrNull() ?: 0L)
    }

    val mask = (0xFFFFFFFFL shl (32 - prefixLen)) and 0xFFFFFFFFL
    val networkValue = value and mask
    return listOf(
      (networkValue shr 24) and 0xFF,
      (networkValue shr 16) and 0xFF,
      (networkValue shr 8) and 0xFF,
      networkValue and 0xFF,
    ).joinToString(".")
  }

  companion object {
    private const val TAG = "VntVpnService"
    private const val NOTIFICATION_CHANNEL_ID = "vnt_vpn_service"
    private const val NOTIFICATION_ID = 1001

    const val ACTION_START = "io.github.cg85712.vnt2.vpn.START"
    const val ACTION_RESTART = "io.github.cg85712.vnt2.vpn.RESTART"
    const val ACTION_STOP = "io.github.cg85712.vnt2.vpn.STOP"

    const val EXTRA_FILE_NAME = "file_name"
    const val EXTRA_CONFIG_NAME = "config_name"
    const val EXTRA_CONFIG_JSON = "config_json"

    fun startIntent(context: Context, request: VntLaunchRequest, restart: Boolean): Intent {
      return Intent(context, VntVpnService::class.java)
        .setAction(if (restart) ACTION_RESTART else ACTION_START)
        .putExtra(EXTRA_FILE_NAME, request.fileName)
        .putExtra(EXTRA_CONFIG_NAME, request.configName)
        .putExtra(EXTRA_CONFIG_JSON, request.configJson)
    }
  }

  private fun invalidateSessions() {
    synchronized(sessionLock) {
      sessionGeneration += 1
    }
  }

  private fun nextGeneration(): Int {
    synchronized(sessionLock) {
      sessionGeneration += 1
      return sessionGeneration
    }
  }

  private fun isActive(generation: Int): Boolean {
    synchronized(sessionLock) {
      return sessionGeneration == generation
    }
  }

  private fun ensureActive(generation: Int) {
    if (!isActive(generation)) {
      throw IllegalStateException("启动已取消")
    }
  }
}
