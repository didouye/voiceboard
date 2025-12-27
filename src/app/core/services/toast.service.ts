import { Injectable, signal } from '@angular/core';

export interface Toast {
  id: string;
  message: string;
  action?: {
    label: string;
    callback: () => void;
  };
  duration?: number;
}

@Injectable({ providedIn: 'root' })
export class ToastService {
  toasts = signal<Toast[]>([]);

  show(toast: Omit<Toast, 'id'>): string {
    const id = crypto.randomUUID();
    this.toasts.update(t => [...t, { ...toast, id }]);

    if (toast.duration !== 0) {
      setTimeout(() => this.dismiss(id), toast.duration ?? 10000);
    }

    return id;
  }

  dismiss(id: string): void {
    this.toasts.update(t => t.filter(toast => toast.id !== id));
  }
}
