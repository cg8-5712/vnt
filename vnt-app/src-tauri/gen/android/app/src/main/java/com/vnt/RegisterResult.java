package com.vnt;

import org.json.JSONObject;

public final class RegisterResult {
  private final String ip;
  private final int prefixLen;
  private final String gateway;
  private final String broadcast;

  private RegisterResult(String ip, int prefixLen, String gateway, String broadcast) {
    this.ip = ip;
    this.prefixLen = prefixLen;
    this.gateway = gateway;
    this.broadcast = broadcast;
  }

  static RegisterResult fromJson(String json) throws VntException {
    try {
      JSONObject object = new JSONObject(json);
      if (!object.getBoolean("success")) {
        throw new VntException("Registration failed: " + object.optString("error", "unknown"));
      }

      return new RegisterResult(
          object.getString("ip"),
          object.getInt("prefix_len"),
          object.getString("gateway"),
          object.getString("broadcast"));
    } catch (VntException exception) {
      throw exception;
    } catch (Exception exception) {
      throw new VntException("Failed to parse register result", exception);
    }
  }

  public String getIp() {
    return ip;
  }

  public int getPrefixLen() {
    return prefixLen;
  }

  public String getGateway() {
    return gateway;
  }

  public String getBroadcast() {
    return broadcast;
  }
}
