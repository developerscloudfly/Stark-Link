import { Monitor, Laptop, Smartphone, Tablet, Wifi, WifiOff, Battery } from "lucide-react";
import type { DiscoveredDevice } from "../types";

interface DeviceCardProps {
  device: DiscoveredDevice;
  onConnect: (device: DiscoveredDevice) => void;
}

function getDeviceIcon(deviceType: string) {
  switch (deviceType.toLowerCase()) {
    case "laptop":
      return Laptop;
    case "phone":
      return Smartphone;
    case "tablet":
      return Tablet;
    default:
      return Monitor;
  }
}

function getOsLabel(os: string): string {
  switch (os) {
    case "Windows":
      return "Windows";
    case "macOS":
      return "macOS";
    case "Linux":
      return "Linux";
    case "Android":
      return "Android";
    case "iOS":
      return "iOS";
    default:
      return os;
  }
}

function DeviceCard({ device, onConnect }: DeviceCardProps) {
  const Icon = getDeviceIcon(device.device_type);

  return (
    <div
      className="bg-dark-card border border-dark-border rounded-xl p-5 hover:border-accent-blue/40 transition-all duration-200 cursor-pointer group"
      onClick={() => onConnect(device)}
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-4">
        <div className="w-11 h-11 rounded-lg bg-gradient-to-br from-accent-gradient-start/20 to-accent-gradient-end/20 flex items-center justify-center">
          <Icon size={22} className="text-accent-blue" />
        </div>
        <div className="flex items-center gap-1.5">
          {device.online ? (
            <>
              <Wifi size={14} className="text-status-online" />
              <span className="text-xs text-status-online font-medium">Online</span>
            </>
          ) : (
            <>
              <WifiOff size={14} className="text-status-offline" />
              <span className="text-xs text-status-offline font-medium">Offline</span>
            </>
          )}
        </div>
      </div>

      {/* Info */}
      <h3 className="text-sm font-semibold text-dark-text mb-1 group-hover:text-white transition-colors">
        {device.name}
      </h3>
      <p className="text-xs text-dark-text-secondary mb-3">
        {getOsLabel(device.os)} &middot; {device.device_type}
      </p>

      {/* Battery */}
      {device.battery_level != null && (
        <div className="flex items-center gap-1.5">
          <Battery size={14} className="text-dark-text-secondary" />
          <span className="text-xs text-dark-text-secondary">
            {device.battery_level}%
          </span>
        </div>
      )}

      {/* Address */}
      {device.addresses.length > 0 && (
        <p className="text-[11px] text-dark-text-secondary mt-2 font-mono">
          {device.addresses[0]}:{device.port}
        </p>
      )}
    </div>
  );
}

export default DeviceCard;
