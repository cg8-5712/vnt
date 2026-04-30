package com.vnt;

import android.net.VpnService;

public final class VntManager {
  static {
    System.loadLibrary("vnt_app_lib");
  }

  private VntManager() {}

  public static boolean init() {
    return nativeInit();
  }

  public static void destroy() {
    nativeDestroy();
  }

  public static VntNetwork createNetwork(String configJson) {
    long handle = nativeCreateNetwork(configJson);
    if (handle < 0) {
      return null;
    }
    return new VntNetwork(handle);
  }

  public static void setSocketProtector(VpnService vpnService) {
    nativeSetSocketProtector(vpnService);
  }

  public static void clearSocketProtector() {
    nativeSetSocketProtector(null);
  }

  private static native boolean nativeInit();

  private static native void nativeDestroy();

  private static native long nativeCreateNetwork(String configJson);

  private static native void nativeSetSocketProtector(Object protector);
}
