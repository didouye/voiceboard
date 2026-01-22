import { Component, Input } from "@angular/core";
import { CommonModule } from "@angular/common";

@Component({
  selector: "app-vu-meter",
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="h-1.5 bg-surface rounded-full overflow-hidden">
      <div
        class="h-full rounded-full transition-[width] duration-50 ease-out"
        [style.width.%]="displayLevel"
        [style.background]="gradient"
      ></div>
    </div>
  `,
  styles: [],
})
export class VuMeterComponent {
  @Input() level = 0; // 0-1

  readonly gradient = "linear-gradient(to right, #22c55e, #eab308, #ef4444)";

  // Amplify the level for better visibility
  // Only show if level is above noise floor threshold
  get displayLevel(): number {
    // Noise floor threshold - ignore very small values (background noise)
    if (this.level < 0.01) return 0;
    // Amplify for visibility (x2) and cap at 100%
    return Math.min(100, this.level * 200);
  }
}
