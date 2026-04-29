package io.github.cg85712.vnt2.vpn

import android.app.Activity
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.util.Log
import androidx.activity.result.ActivityResult
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@InvokeArg
class StartArgs {
  lateinit var fileName: String
  lateinit var configName: String
  lateinit var configJson: String
}

class VntVpnPlugin(private val activity: Activity) : Plugin(activity) {
  private var pendingRequest: VntLaunchRequest? = null
  private var pendingRestart: Boolean = false

  @Command
  fun start(invoke: Invoke) {
    Log.i(TAG, "start command received from frontend")
    handleStart(invoke, restart = false)
  }

  @Command
  fun restart(invoke: Invoke) {
    Log.i(TAG, "restart command received from frontend")
    handleStart(invoke, restart = true)
  }

  @Command
  fun stop(invoke: Invoke) {
    Log.i(TAG, "stop command received from frontend")
    val intent = Intent(activity, VntVpnService::class.java).setAction(VntVpnService.ACTION_STOP)
    activity.startService(intent)
    invoke.resolve()
  }

  @Command
  fun startStatus(invoke: Invoke) {
    invoke.resolveObject(VntVpnRuntime.snapshotStartStatus())
  }

  @Command
  fun info(invoke: Invoke) {
    invoke.resolveObject(VntVpnRuntime.snapshotInfo())
  }

  @Command
  fun peers(invoke: Invoke) {
    invoke.resolveObject(VntVpnRuntime.snapshotPeers())
  }

  @Command
  fun routes(invoke: Invoke) {
    invoke.resolveObject(VntVpnRuntime.snapshotRoutes())
  }

  @ActivityCallback
  fun handleVpnPermissionResult(invoke: Invoke, result: ActivityResult) {
    val request = pendingRequest
    val restart = pendingRestart
    pendingRequest = null
    pendingRestart = false

    if (result.resultCode != Activity.RESULT_OK || request == null) {
      VntVpnRuntime.markStartFailed("VPN 权限被拒绝")
      invoke.reject("VPN permission denied")
      return
    }

    launchService(request, restart)
    invoke.resolve()
  }

  private fun handleStart(invoke: Invoke, restart: Boolean) {
    val args = invoke.parseArgs(StartArgs::class.java)
    val request = VntLaunchRequest(args.fileName, args.configName, args.configJson)
    val config = VntVpnRuntime.parseLaunchConfig(request)

    val currentStatus = VntVpnRuntime.snapshotStartStatus().status
    if (!restart && currentStatus != "stopped") {
      invoke.reject("VNT is already starting or running")
      return
    }

    val prepareIntent = VpnService.prepare(activity)
    pendingRequest = request
    pendingRestart = restart
    VntVpnRuntime.beginStart(
      request,
      config,
      if (prepareIntent == null) "正在提交启动请求" else "等待 VPN 授权",
    )

    if (prepareIntent != null) {
      startActivityForResult(invoke, prepareIntent, "handleVpnPermissionResult")
      return
    }

    launchService(request, restart)
    pendingRequest = null
    pendingRestart = false
    invoke.resolve()
  }

  private fun launchService(request: VntLaunchRequest, restart: Boolean) {
    Log.i(TAG, "launching VPN service restart=$restart file=${request.fileName}")
    VntVpnRuntime.persistRequest(activity, request)
    val intent = VntVpnService.startIntent(activity, request, restart)
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      activity.startForegroundService(intent)
    } else {
      @Suppress("DEPRECATION")
      activity.startService(intent)
    }
  }

  companion object {
    private const val TAG = "VntVpnPlugin"
  }
}
