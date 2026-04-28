package io.github.cg85712.vnt2.vpn

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
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
    return when (intent?.action ?: ACTION_START) {
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
      try {
        releaseHandles()
        ensureActive(generation)

        VntVpnRuntime.appendLog("正在初始化 VNT")
        if (!VntManager.init()) {
          throw IllegalStateException("无法初始化 VNT")
        }
        ensureActive(generation)

        localNetwork = VntManager.createNetwork(request.configJson)
          ?: throw IllegalStateException("无法创建网络实例")
        ensureActive(generation)

        VntVpnRuntime.appendLog("正在注册网络")
        val registerResult = localNetwork.register()
        ensureActive(generation)
        VntVpnRuntime.appendLog("已获取地址 ${registerResult.ip}/${registerResult.prefixLen}")

        localVpnInterface = establishVpn(config, registerResult)
        ensureActive(generation)
        VntVpnRuntime.appendLog("VPN 接口已建立")

        localNetwork.startTun(localVpnInterface.fd)
        ensureActive(generation)
        val api = localNetwork.getApi()

        synchronized(sessionLock) {
          network = localNetwork
          vpnInterface = localVpnInterface
        }

        VntVpnRuntime.markRunning(localNetwork, api)
        updateNotification("VNT 已连接", "${registerResult.ip}/${registerResult.prefixLen}")
      } catch (exception: Exception) {
        try {
          localNetwork?.stop()
        } catch (_: Exception) {
        }
        try {
          localVpnInterface?.close()
        } catch (_: Exception) {
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
    val builder = Builder()
      .setSession("VNT2")
      .setMtu(config.mtu)
      .setConfigureIntent(createOpenAppPendingIntent())
      .addAddress(registerResult.ip, registerResult.prefixLen)

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
      try {
        network?.stop()
      } catch (_: Exception) {
      }
      network = null

      try {
        vpnInterface?.close()
      } catch (_: Exception) {
      }
      vpnInterface = null
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
