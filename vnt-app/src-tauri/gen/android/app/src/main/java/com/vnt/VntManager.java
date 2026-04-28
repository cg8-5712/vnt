package com.vnt;

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

  private static native boolean nativeInit();

  private static native void nativeDestroy();

  private static native long nativeCreateNetwork(String configJson);
}
