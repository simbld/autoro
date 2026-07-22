//!  To avoid robotic repetition, you can have 2-3 variations of the spark and draw one at random:
//   private sparkVariants = [
//     new Audio('/sounds/spark-1.mp3'),
//     new Audio('/sounds/halogen.mp3'),
//     new Audio('/sounds/spark-3.mp3'),
//   ];
//
//   toggle() {
//     this.isOn = !this.isOn;
//     if (this.isOn) {
//       const rand = this.sparkVariants[Math.floor(Math.random() * this.sparkVariants.length)];
//       rand.currentTime = 0;
//       rand.play();
//     }
//   }
//!
import {Component, Input} from "@angular/core";

@Component({
  selector: 'app-plug',
  imports: [],
  templateUrl: './plug.component.html',
  styleUrl: './plug.component.scss'
})
export class PlugComponent {
    isOn = false;
    @Input()
    collapsed = false;

    private halogen: HTMLAudioElement = new Audio('/sounds/halogen.mp3');
    private unplug: HTMLAudioElement = new Audio('/sounds/unplug.mp3');

    toggle() {
        this.isOn = !this.isOn;
        
        if (this.isOn) {
            this.halogen.currentTime = 0;
            this.halogen.play().catch(err => console.warn('Error playing audio:', err));
        } else {
            this.unplug.currentTime = 0;
            this.unplug.play().catch(err => console.warn('Error playing audio:', err));
        }
    }
}
