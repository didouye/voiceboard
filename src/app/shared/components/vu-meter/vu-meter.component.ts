import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-vu-meter',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="h-1.5 bg-surface rounded-full overflow-hidden">
      <div
        class="h-full rounded-full transition-[width] duration-50 ease-out"
        [style.width.%]="level * 100"
        [style.background]="gradient"
      ></div>
    </div>
  `,
  styles: []
})
export class VuMeterComponent {
  @Input() level = 0; // 0-1

  readonly gradient = 'linear-gradient(to right, #22c55e, #eab308, #ef4444)';
}
