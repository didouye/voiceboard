/**
 * Audio device model matching the Rust AudioDevice struct
 */
export interface AudioDevice {
  id: string;
  name: string;
  device_type: 'input' | 'output';
  is_default: boolean;
}

/**
 * Device type enum
 */
export enum DeviceType {
  Input = 'input',
  Output = 'output'
}
