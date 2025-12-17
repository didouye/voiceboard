import { Component } from '@angular/core';
import { MixerComponent } from './features/mixer/mixer.component';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [MixerComponent],
  template: `<app-mixer />`,
  styles: [`
    :host {
      display: block;
      min-height: 100vh;
    }
  `]
})
export class AppComponent {}
