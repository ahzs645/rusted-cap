/**
 * Cap Electron Capture Library TypeScript Definitions
 */

export interface PlatformCapabilities {
  platform: Platform;
  audio: AudioCapabilities;
  screen: ScreenCapabilities;
  systemVersion: string;
  permissions: PermissionStatus;
}

export enum Platform {
  MacOS = 'MacOS',
  Windows = 'Windows',
  Linux = 'Linux',
  Unknown = 'Unknown'
}

export interface AudioCapabilities {
  systemAudio: boolean;
  microphone: boolean;
  sampleRates: number[];
  formats: string[];
  inputDevices: number;
  outputDevices: number;
}

export interface ScreenCapabilities {
  supported: boolean;
  displayCount: number;
  windowCapture: boolean;
  maxResolution?: [number, number];
  frameRates: number[];
}

export interface PermissionStatus {
  microphone: PermissionState;
  screenRecording: PermissionState;
  systemAudio: PermissionState;
}

export enum PermissionState {
  Granted = 'Granted',
  Denied = 'Denied',
  NotRequested = 'NotRequested',
  Requesting = 'Requesting',
  NotApplicable = 'NotApplicable'
}

export interface AudioDevice {
  id: string;
  name: string;
  deviceType: AudioDeviceType;
  isDefault: boolean;
  sampleRates: number[];
  channels: number[];
}

export enum AudioDeviceType {
  Input = 'Input',
  Output = 'Output'
}

export interface Display {
  id: number;
  name: string;
  resolution: [number, number];
  position: [number, number];
  isPrimary: boolean;
  scaleFactor: number;
}

export interface Window {
  id: number;
  title: string;
  appName: string;
  bounds: [number, number, number, number]; // x, y, width, height
  isMinimized: boolean;
  isVisible: boolean;
}

export interface CaptureConfig {
  audio: AudioCaptureConfig;
  screen: ScreenCaptureConfig;
  output: OutputFormat;
}

export interface AudioCaptureConfig {
  enabled: boolean;
  systemAudio: boolean;
  microphone: boolean;
  sampleRate: number;
  channels: number;
  segmentDurationMs: number;
  microphoneDeviceId?: string;
  format: AudioFormat;
}

export interface ScreenCaptureConfig {
  enabled: boolean;
  displayId?: number;
  fps: number;
  quality: number;
  includeCursor: boolean;
  windowId?: number;
}

export interface OutputFormat {
  audio: AudioFormat;
  video: VideoFormat;
  outputDir?: string;
  realTime: boolean;
}

export enum AudioFormat {
  Aac = 'Aac',
  Mp3 = 'Mp3',
  Wav = 'Wav',
  Raw = 'Raw'
}

export enum VideoFormat {
  Mp4 = 'Mp4',
  WebM = 'WebM',
  Raw = 'Raw'
}

export interface AudioSegment {
  data: ArrayBuffer;
  sampleRate: number;
  channels: number;
  timestamp: number;
  durationMs: number;
  source: AudioSource;
}

export enum AudioSource {
  Microphone = 'Microphone',
  SystemAudio = 'SystemAudio',
  Mixed = 'Mixed'
}

export interface ScreenFrame {
  data: ArrayBuffer;
  width: number;
  height: number;
  timestamp: number;
  frameNumber: number;
}

export declare class CaptureSession {
  constructor(config: CaptureConfig);
  
  start(): Promise<void>;
  stop(): Promise<void>;
  isActive(): Promise<boolean>;
}

/**
 * Initialize the library and check platform capabilities
 */
export declare function init(): string;

/**
 * Get available audio devices
 */
export declare function getAudioDevices(): string;

/**
 * Get available displays for screen capture
 */
export declare function getDisplays(): string;

/**
 * Request all necessary permissions for audio and screen capture
 */
export declare function requestPermissions(): Promise<string>;

/**
 * Check current permission status without requesting
 */
export declare function checkPermissions(): Promise<string>;

/**
 * Get platform-specific instructions for enabling system audio capture
 */
export declare function getSystemAudioSetupInstructions(): string;

/**
 * Create a new capture session with configuration
 */
export declare function createCaptureSession(config?: Partial<CaptureConfig>): CaptureSession;

// Re-export for convenience
export {
  AudioFormat,
  VideoFormat,
  AudioSource,
  PermissionState,
  Platform
};
