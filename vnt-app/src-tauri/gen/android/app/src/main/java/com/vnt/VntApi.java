package com.vnt;

import java.util.ArrayList;
import java.util.List;
import org.json.JSONArray;
import org.json.JSONObject;

public final class VntApi {
  private final long nativeHandle;

  VntApi(long nativeHandle) {
    this.nativeHandle = nativeHandle;
  }

  public List<ClientInfo> getClientList() throws VntException {
    try {
      JSONArray array = new JSONArray(nativeGetClientList(nativeHandle));
      List<ClientInfo> items = new ArrayList<>();
      for (int index = 0; index < array.length(); index++) {
        JSONObject object = array.getJSONObject(index);
        items.add(new ClientInfo(object.getString("ip"), object.getBoolean("online")));
      }
      return items;
    } catch (Exception exception) {
      throw new VntException("Failed to get client list", exception);
    }
  }

  public NetworkInfo getNetwork() throws VntException {
    try {
      String json = nativeGetNetwork(nativeHandle);
      if ("null".equals(json)) {
        return null;
      }

      JSONObject object = new JSONObject(json);
      return new NetworkInfo(
          object.getString("ip"),
          object.getInt("prefix_len"),
          object.getString("gateway"),
          object.getString("broadcast"));
    } catch (Exception exception) {
      throw new VntException("Failed to get network info", exception);
    }
  }

  public NatInfo getNatInfo() throws VntException {
    try {
      String json = nativeGetNatInfo(nativeHandle);
      if ("null".equals(json)) {
        return null;
      }
      return NatInfo.fromJson(json);
    } catch (Exception exception) {
      throw new VntException("Failed to get NAT info", exception);
    }
  }

  public List<ServerInfo> getServerList() throws VntException {
    try {
      JSONArray array = new JSONArray(nativeGetServerList(nativeHandle));
      List<ServerInfo> items = new ArrayList<>();
      for (int index = 0; index < array.length(); index++) {
        JSONObject object = array.getJSONObject(index);
        items.add(
            new ServerInfo(
                object.getInt("server_id"),
                object.getString("server_addr"),
                object.getBoolean("connected"),
                object.isNull("rtt") ? null : object.getInt("rtt"),
                object.getLong("data_version"),
                object.isNull("server_version") ? null : object.getString("server_version")));
      }
      return items;
    } catch (Exception exception) {
      throw new VntException("Failed to get server list", exception);
    }
  }

  public List<RouteInfo> getRouteTable() throws VntException {
    try {
      JSONArray array = new JSONArray(nativeGetRouteTable(nativeHandle));
      List<RouteInfo> items = new ArrayList<>();
      for (int index = 0; index < array.length(); index++) {
        JSONObject object = array.getJSONObject(index);
        JSONArray routesArray = object.getJSONArray("routes");
        List<RouteDetail> routes = new ArrayList<>();
        for (int routeIndex = 0; routeIndex < routesArray.length(); routeIndex++) {
          JSONObject route = routesArray.getJSONObject(routeIndex);
          routes.add(
              new RouteDetail(
                  route.getString("route_key"),
                  route.getString("protocol"),
                  route.getInt("metric"),
                  route.getInt("rtt"),
                  route.optDouble("loss_rate", 0D)));
        }
        items.add(new RouteInfo(object.getString("ip"), routes));
      }
      return items;
    } catch (Exception exception) {
      throw new VntException("Failed to get route table", exception);
    }
  }

  public boolean isDirect(String ip) {
    return nativeIsDirect(nativeHandle, ip);
  }

  public NatInfo getPeerNatInfo(String ip) throws VntException {
    try {
      String json = nativeGetPeerNatInfo(nativeHandle, ip);
      if ("null".equals(json)) {
        return null;
      }
      return NatInfo.fromJson(json);
    } catch (Exception exception) {
      throw new VntException("Failed to get peer NAT info", exception);
    }
  }

  public PacketLossInfo getPacketLoss(String ip) throws VntException {
    try {
      String json = nativeGetPacketLoss(nativeHandle, ip);
      if ("null".equals(json)) {
        return null;
      }

      JSONObject object = new JSONObject(json);
      return new PacketLossInfo(
          object.getString("ip"),
          object.getLong("sent"),
          object.getLong("received"),
          object.getDouble("loss_rate"));
    } catch (Exception exception) {
      throw new VntException("Failed to get packet loss", exception);
    }
  }

  public TrafficInfo getTrafficInfo(String ip) throws VntException {
    try {
      String json = nativeGetTrafficInfo(nativeHandle, ip);
      if ("null".equals(json)) {
        return null;
      }

      JSONObject object = new JSONObject(json);
      return new TrafficInfo(
          object.getString("ip"), object.getLong("tx_bytes"), object.getLong("rx_bytes"));
    } catch (Exception exception) {
      throw new VntException("Failed to get traffic info", exception);
    }
  }

  private static native String nativeGetClientList(long apiHandle);

  private static native String nativeGetNetwork(long apiHandle);

  private static native String nativeGetNatInfo(long apiHandle);

  private static native String nativeGetServerList(long apiHandle);

  private static native String nativeGetRouteTable(long apiHandle);

  private static native boolean nativeIsDirect(long apiHandle, String ip);

  private static native String nativeGetPeerNatInfo(long apiHandle, String ip);

  private static native String nativeGetPacketLoss(long apiHandle, String ip);

  private static native String nativeGetTrafficInfo(long apiHandle, String ip);

  public static final class ClientInfo {
    private final String ip;
    private final boolean online;

    public ClientInfo(String ip, boolean online) {
      this.ip = ip;
      this.online = online;
    }

    public String getIp() {
      return ip;
    }

    public boolean isOnline() {
      return online;
    }
  }

  public static final class NetworkInfo {
    private final String ip;
    private final int prefixLen;
    private final String gateway;
    private final String broadcast;

    public NetworkInfo(String ip, int prefixLen, String gateway, String broadcast) {
      this.ip = ip;
      this.prefixLen = prefixLen;
      this.gateway = gateway;
      this.broadcast = broadcast;
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

  public static final class NatInfo {
    private final String natType;
    private final List<String> publicIps;
    private final String ipv6;

    private NatInfo(String natType, List<String> publicIps, String ipv6) {
      this.natType = natType;
      this.publicIps = publicIps;
      this.ipv6 = ipv6;
    }

    static NatInfo fromJson(String json) throws Exception {
      JSONObject object = new JSONObject(json);
      JSONArray ipsArray = object.getJSONArray("public_ips");
      List<String> publicIps = new ArrayList<>();
      for (int index = 0; index < ipsArray.length(); index++) {
        publicIps.add(ipsArray.getString(index));
      }
      return new NatInfo(
          object.getString("nat_type"),
          publicIps,
          object.isNull("ipv6") ? null : object.getString("ipv6"));
    }

    public String getNatType() {
      return natType;
    }

    public List<String> getPublicIps() {
      return publicIps;
    }

    public String getIpv6() {
      return ipv6;
    }
  }

  public static final class ServerInfo {
    private final int serverId;
    private final String serverAddr;
    private final boolean connected;
    private final Integer rtt;
    private final long dataVersion;
    private final String serverVersion;

    public ServerInfo(
        int serverId,
        String serverAddr,
        boolean connected,
        Integer rtt,
        long dataVersion,
        String serverVersion) {
      this.serverId = serverId;
      this.serverAddr = serverAddr;
      this.connected = connected;
      this.rtt = rtt;
      this.dataVersion = dataVersion;
      this.serverVersion = serverVersion;
    }

    public int getServerId() {
      return serverId;
    }

    public String getServerAddr() {
      return serverAddr;
    }

    public boolean isConnected() {
      return connected;
    }

    public Integer getRtt() {
      return rtt;
    }

    public long getDataVersion() {
      return dataVersion;
    }

    public String getServerVersion() {
      return serverVersion;
    }
  }

  public static final class RouteInfo {
    private final String ip;
    private final List<RouteDetail> routes;

    public RouteInfo(String ip, List<RouteDetail> routes) {
      this.ip = ip;
      this.routes = routes;
    }

    public String getIp() {
      return ip;
    }

    public List<RouteDetail> getRoutes() {
      return routes;
    }
  }

  public static final class RouteDetail {
    private final String routeKey;
    private final String protocol;
    private final int metric;
    private final int rtt;
    private final double lossRate;

    public RouteDetail(String routeKey, String protocol, int metric, int rtt, double lossRate) {
      this.routeKey = routeKey;
      this.protocol = protocol;
      this.metric = metric;
      this.rtt = rtt;
      this.lossRate = lossRate;
    }

    public String getRouteKey() {
      return routeKey;
    }

    public String getProtocol() {
      return protocol;
    }

    public int getMetric() {
      return metric;
    }

    public int getRtt() {
      return rtt;
    }

    public double getLossRate() {
      return lossRate;
    }
  }

  public static final class PacketLossInfo {
    private final String ip;
    private final long sent;
    private final long received;
    private final double lossRate;

    public PacketLossInfo(String ip, long sent, long received, double lossRate) {
      this.ip = ip;
      this.sent = sent;
      this.received = received;
      this.lossRate = lossRate;
    }

    public String getIp() {
      return ip;
    }

    public long getSent() {
      return sent;
    }

    public long getReceived() {
      return received;
    }

    public double getLossRate() {
      return lossRate;
    }
  }

  public static final class TrafficInfo {
    private final String ip;
    private final long txBytes;
    private final long rxBytes;

    public TrafficInfo(String ip, long txBytes, long rxBytes) {
      this.ip = ip;
      this.txBytes = txBytes;
      this.rxBytes = rxBytes;
    }

    public String getIp() {
      return ip;
    }

    public long getTxBytes() {
      return txBytes;
    }

    public long getRxBytes() {
      return rxBytes;
    }
  }
}
