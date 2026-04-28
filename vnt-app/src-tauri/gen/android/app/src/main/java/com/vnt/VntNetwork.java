package com.vnt;

public final class VntNetwork {
  private final long nativeHandle;
  private boolean closed = false;

  VntNetwork(long nativeHandle) {
    this.nativeHandle = nativeHandle;
  }

  public RegisterResult register() throws VntException {
    ensureOpen();
    return RegisterResult.fromJson(nativeRegister(nativeHandle));
  }

  public void startTun(int tunFd) throws VntException {
    ensureOpen();
    if (!nativeStartTun(nativeHandle, tunFd)) {
      throw new VntException("Failed to start TUN device");
    }
  }

  public VntApi getApi() throws VntException {
    ensureOpen();
    long apiHandle = nativeGetApi(nativeHandle);
    if (apiHandle < 0) {
      throw new VntException("Failed to get VNT API");
    }
    return new VntApi(apiHandle);
  }

  public boolean isNoTun() {
    ensureOpen();
    return nativeIsNoTun(nativeHandle);
  }

  public void stop() {
    if (closed) {
      return;
    }

    nativeStop(nativeHandle);
    closed = true;
  }

  private void ensureOpen() {
    if (closed) {
      throw new IllegalStateException("VntNetwork is already closed");
    }
  }

  private static native String nativeRegister(long handle);

  private static native boolean nativeStartTun(long handle, int tunFd);

  private static native long nativeGetApi(long handle);

  private static native boolean nativeIsNoTun(long handle);

  private static native boolean nativeStop(long handle);
}
