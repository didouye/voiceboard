/**
 * Audio device service - handles device management
 */
import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { TauriService } from './tauri.service';
import { AudioDevice } from '../models/audio-device.model';

@Injectable({
  providedIn: 'root'
})
export class AudioDeviceService {
  private inputDevicesSubject = new BehaviorSubject<AudioDevice[]>([]);
  private outputDevicesSubject = new BehaviorSubject<AudioDevice[]>([]);
  private selectedInputDeviceSubject = new BehaviorSubject<AudioDevice | null>(null);
  private selectedOutputDeviceSubject = new BehaviorSubject<AudioDevice | null>(null);

  public inputDevices$: Observable<AudioDevice[]> = this.inputDevicesSubject.asObservable();
  public outputDevices$: Observable<AudioDevice[]> = this.outputDevicesSubject.asObservable();
  public selectedInputDevice$: Observable<AudioDevice | null> = this.selectedInputDeviceSubject.asObservable();
  public selectedOutputDevice$: Observable<AudioDevice | null> = this.selectedOutputDeviceSubject.asObservable();

  constructor(private tauri: TauriService) {
    this.loadDevices();
  }

  /**
   * Load all audio devices
   */
  async loadDevices(): Promise<void> {
    try {
      const [inputDevices, outputDevices, defaultInput, defaultOutput] = await Promise.all([
        this.tauri.invoke<AudioDevice[]>('get_input_devices'),
        this.tauri.invoke<AudioDevice[]>('get_output_devices'),
        this.tauri.invoke<AudioDevice | null>('get_default_input_device'),
        this.tauri.invoke<AudioDevice | null>('get_default_output_device')
      ]);

      this.inputDevicesSubject.next(inputDevices);
      this.outputDevicesSubject.next(outputDevices);

      if (defaultInput) {
        this.selectedInputDeviceSubject.next(defaultInput);
      }

      if (defaultOutput) {
        this.selectedOutputDeviceSubject.next(defaultOutput);
      }
    } catch (error) {
      console.error('Failed to load devices:', error);
      throw error;
    }
  }

  /**
   * Get input devices
   */
  async getInputDevices(): Promise<AudioDevice[]> {
    try {
      const devices = await this.tauri.invoke<AudioDevice[]>('get_input_devices');
      this.inputDevicesSubject.next(devices);
      return devices;
    } catch (error) {
      console.error('Failed to get input devices:', error);
      throw error;
    }
  }

  /**
   * Get output devices
   */
  async getOutputDevices(): Promise<AudioDevice[]> {
    try {
      const devices = await this.tauri.invoke<AudioDevice[]>('get_output_devices');
      this.outputDevicesSubject.next(devices);
      return devices;
    } catch (error) {
      console.error('Failed to get output devices:', error);
      throw error;
    }
  }

  /**
   * Select an input device
   */
  async selectInputDevice(device: AudioDevice): Promise<void> {
    try {
      await this.tauri.invoke('select_input_device', { deviceId: device.id });
      this.selectedInputDeviceSubject.next(device);
    } catch (error) {
      console.error('Failed to select input device:', error);
      throw error;
    }
  }

  /**
   * Select an output device
   */
  async selectOutputDevice(device: AudioDevice): Promise<void> {
    try {
      await this.tauri.invoke('select_output_device', { deviceId: device.id });
      this.selectedOutputDeviceSubject.next(device);
    } catch (error) {
      console.error('Failed to select output device:', error);
      throw error;
    }
  }
}
