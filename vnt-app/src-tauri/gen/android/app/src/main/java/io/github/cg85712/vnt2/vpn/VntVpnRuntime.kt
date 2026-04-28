package io.github.cg85712.vnt2.vpn

import android.content.Context
import android.os.Build
import com.vnt.VntApi
import com.vnt.VntNetwork
import io.github.cg85712.vnt2.BuildConfig
import org.json.JSONObject
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

data class VntLaunchRequest(
  val fileName: String,
  val configName: String,
  val configJson: String,
)

data class VntLaunchConfig(
  val fileName: String,
  val configName: String,
  val configJson: String,
  val deviceId: String,
  val deviceName: String,
  val networkCode: String,
  val mtu: Int,
  val fec: Boolean,
  val compress: Boolean,
  val encrypt: Boolean,
  val rtx: Boolean,
  val outputRoutes: List<String>,
)

data class StartStatusPayload(
  val status: String,
  val logs: List<String>,
)

data class ServerInfoPayload(
  val server: String,
  val connected: Boolean,
  val server_rtt: Int?,
  val server_version: String?,
)

data class AppInfoPayload(
  val name: String,
  val version: String,
  val ip: String?,
  val prefix_len: Int?,
  val gateway: String?,
  val device_id: String,
  val status: String,
  val current_config_name: String?,
  val current_config_file: String?,
  val online_client_num: Int,
  val offline_client_num: Int,
  val direct_client_num: Int,
  val server_info: List<ServerInfoPayload>,
  val nat_type: String?,
  val public_ipv6: String?,
  val public_ipv4s: List<String>,
  val network_code: String?,
  val mtu: Int?,
  val fec: Boolean?,
  val compress: Boolean?,
  val encrypt: Boolean?,
  val rtx: Boolean?,
)

data class PeerNatInfoPayload(
  val nat_type: String,
  val public_ips: List<String>,
  val ipv6: String?,
)

data class PacketLossPayload(
  val sent: Long,
  val received: Long,
  val loss_rate: Double,
)

data class TrafficPayload(
  val tx_bytes: Long,
  val rx_bytes: Long,
)

data class RouteDetailPayload(
  val addr: String,
  val protocol: String,
  val metric: Int,
  val rtt: Int,
  val loss_rate: Double,
)

data class PeerItemPayload(
  val ip: String,
  val name: String?,
  val online: Boolean,
  val route: RouteDetailPayload?,
  val version: String,
  val last_connected_time: Long,
  val key_equal: Int,
  val nat_info: PeerNatInfoPayload?,
  val packet_loss: PacketLossPayload?,
  val traffic: TrafficPayload?,
)

data class RouteItemPayload(
  val ip: String,
  val routes: List<RouteDetailPayload>,
)

object VntVpnRuntime {
  private const val PREFS_NAME = "vnt_vpn_runtime"
  private const val KEY_FILE_NAME = "file_name"
  private const val KEY_CONFIG_NAME = "config_name"
  private const val KEY_CONFIG_JSON = "config_json"
  private const val DEFAULT_MTU = 1380

  private val lock = Any()
  private val timeFormat = SimpleDateFormat("HH:mm:ss", Locale.US)

  private var status: String = "stopped"
  private val startLogs = mutableListOf<String>()
  private var currentRequest: VntLaunchRequest? = null
  private var currentConfig: VntLaunchConfig? = null
  private var network: VntNetwork? = null
  private var api: VntApi? = null

  fun parseLaunchConfig(request: VntLaunchRequest): VntLaunchConfig {
    val json = JSONObject(request.configJson)
    val outputArray = json.optJSONArray("output")
    val outputRoutes = mutableListOf<String>()
    if (outputArray != null) {
      for (index in 0 until outputArray.length()) {
        outputRoutes.add(outputArray.optString(index))
      }
    }

    val configuredName = json.optString("device_name", "").trim()
    val deviceName = if (configuredName.isNotEmpty()) configuredName else Build.MODEL
    val configuredMtu = if (json.has("mtu") && !json.isNull("mtu")) json.optInt("mtu", DEFAULT_MTU) else DEFAULT_MTU
    return VntLaunchConfig(
      fileName = request.fileName,
      configName = request.configName,
      configJson = request.configJson,
      deviceId = json.optString("device_id", ""),
      deviceName = deviceName,
      networkCode = json.optString("network_code", ""),
      mtu = if (configuredMtu > 0) configuredMtu else DEFAULT_MTU,
      fec = json.optBoolean("fec", false),
      compress = json.optBoolean("compress", false),
      encrypt = json.optString("password", "").isNotBlank(),
      rtx = json.optBoolean("rtx", false),
      outputRoutes = outputRoutes,
    )
  }

  fun persistRequest(context: Context, request: VntLaunchRequest) {
    context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
      .edit()
      .putString(KEY_FILE_NAME, request.fileName)
      .putString(KEY_CONFIG_NAME, request.configName)
      .putString(KEY_CONFIG_JSON, request.configJson)
      .apply()
  }

  fun loadPersistedRequest(context: Context): VntLaunchRequest? {
    val preferences = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    val fileName = preferences.getString(KEY_FILE_NAME, null)
    val configName = preferences.getString(KEY_CONFIG_NAME, null)
    val configJson = preferences.getString(KEY_CONFIG_JSON, null)

    if (fileName.isNullOrBlank() || configName.isNullOrBlank() || configJson.isNullOrBlank()) {
      return null
    }

    return VntLaunchRequest(fileName, configName, configJson)
  }

  fun clearPersistedRequest(context: Context) {
    context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
      .edit()
      .clear()
      .apply()
  }

  fun beginStart(request: VntLaunchRequest, config: VntLaunchConfig, message: String? = null) {
    synchronized(lock) {
      status = "starting"
      currentRequest = request
      currentConfig = config
      startLogs.clear()
      network = null
      api = null
      if (!message.isNullOrBlank()) {
        startLogs.add(timestamped(message))
      }
    }
  }

  fun appendLog(message: String) {
    synchronized(lock) {
      if (status == "starting") {
        startLogs.add(timestamped(message))
      }
    }
  }

  fun markRunning(network: VntNetwork, api: VntApi) {
    synchronized(lock) {
      this.network = network
      this.api = api
      status = "running"
      startLogs.clear()
    }
  }

  fun markStartFailed(message: String) {
    synchronized(lock) {
      startLogs.add(timestamped(message))
      status = "stopped"
      network = null
      api = null
      currentRequest = null
      currentConfig = null
    }
  }

  fun markStopped(message: String? = null) {
    synchronized(lock) {
      if (!message.isNullOrBlank()) {
        startLogs.add(timestamped(message))
      }
      status = "stopped"
      network = null
      api = null
      currentRequest = null
      currentConfig = null
    }
  }

  fun snapshotStartStatus(): StartStatusPayload {
    synchronized(lock) {
      return StartStatusPayload(status = status, logs = startLogs.toList())
    }
  }

  fun snapshotInfo(): AppInfoPayload {
    val localStatus: String
    val localConfig: VntLaunchConfig?
    val localApi: VntApi?
    synchronized(lock) {
      localStatus = status
      localConfig = currentConfig
      localApi = api
    }

    if (localConfig == null) {
      return AppInfoPayload(
        name = "",
        version = BuildConfig.VERSION_NAME,
        ip = null,
        prefix_len = null,
        gateway = null,
        device_id = "",
        status = localStatus,
        current_config_name = null,
        current_config_file = null,
        online_client_num = 0,
        offline_client_num = 0,
        direct_client_num = 0,
        server_info = emptyList(),
        nat_type = null,
        public_ipv6 = null,
        public_ipv4s = emptyList(),
        network_code = null,
        mtu = null,
        fec = null,
        compress = null,
        encrypt = null,
        rtx = null,
      )
    }

    val networkInfo = localApi?.safeCall { getNetwork() }
    val clientList = localApi?.safeCall { getClientList() }.orEmpty()
    val serverList = localApi?.safeCall { getServerList() }.orEmpty()
    val natInfo = localApi?.safeCall { getNatInfo() }
    val directCount = clientList.count { client ->
      try {
        localApi?.isDirect(client.ip) == true
      } catch (_: Exception) {
        false
      }
    }

    return AppInfoPayload(
      name = localConfig.deviceName,
      version = BuildConfig.VERSION_NAME,
      ip = networkInfo?.ip,
      prefix_len = networkInfo?.prefixLen,
      gateway = networkInfo?.gateway,
      device_id = localConfig.deviceId,
      status = localStatus,
      current_config_name = localConfig.configName,
      current_config_file = localConfig.fileName,
      online_client_num = clientList.count { it.isOnline() },
      offline_client_num = clientList.count { !it.isOnline() },
      direct_client_num = directCount,
      server_info = serverList.map {
        ServerInfoPayload(
          server = it.serverAddr,
          connected = it.isConnected(),
          server_rtt = it.rtt,
          server_version = it.serverVersion,
        )
      },
      nat_type = natInfo?.natType,
      public_ipv6 = natInfo?.ipv6,
      public_ipv4s = natInfo?.publicIps ?: emptyList(),
      network_code = localConfig.networkCode,
      mtu = localConfig.mtu,
      fec = localConfig.fec,
      compress = localConfig.compress,
      encrypt = localConfig.encrypt,
      rtx = localConfig.rtx,
    )
  }

  fun snapshotPeers(): List<PeerItemPayload> {
    val localApi: VntApi
    synchronized(lock) {
      localApi = api ?: return emptyList()
    }

    val primaryRouteByIp = mutableMapOf<String, RouteDetailPayload>()
    for (routeItem in snapshotRoutes()) {
      routeItem.routes.firstOrNull()?.let { primaryRouteByIp[routeItem.ip] = it }
    }

    val clients = localApi.safeCall { getClientList() }.orEmpty()
    return clients.sortedBy { it.ip }.map { client ->
      val natInfo = localApi.safeCall { getPeerNatInfo(client.ip) }?.let {
        PeerNatInfoPayload(it.natType, it.publicIps, it.ipv6)
      }
      val packetLoss = localApi.safeCall { getPacketLoss(client.ip) }?.let {
        PacketLossPayload(it.sent, it.received, it.lossRate)
      }
      val traffic = localApi.safeCall { getTrafficInfo(client.ip) }?.let {
        TrafficPayload(it.txBytes, it.rxBytes)
      }

      PeerItemPayload(
        ip = client.ip,
        name = null,
        online = client.isOnline() || primaryRouteByIp.containsKey(client.ip),
        route = primaryRouteByIp[client.ip],
        version = "",
        last_connected_time = 0,
        key_equal = 0,
        nat_info = natInfo,
        packet_loss = packetLoss,
        traffic = traffic,
      )
    }
  }

  fun snapshotRoutes(): List<RouteItemPayload> {
    val localApi: VntApi
    synchronized(lock) {
      localApi = api ?: return emptyList()
    }

    return localApi.safeCall { getRouteTable() }
      .orEmpty()
      .map { routeInfo ->
        RouteItemPayload(
          ip = routeInfo.ip,
          routes = routeInfo.routes.map { route ->
            RouteDetailPayload(
              addr = route.routeKey,
              protocol = route.protocol,
              metric = route.metric,
              rtt = route.rtt,
              loss_rate = route.lossRate,
            )
          },
        )
      }
  }

  private fun timestamped(message: String): String {
    return "[${timeFormat.format(Date())}] $message"
  }

  private fun <T> VntApi.safeCall(block: VntApi.() -> T): T? {
    return try {
      block(this)
    } catch (_: Exception) {
      null
    }
  }
}
