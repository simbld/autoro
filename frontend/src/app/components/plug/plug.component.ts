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
    private halogen:  = new Audio('/sounds/');

    toggle() {
        this.isOn = !this.isOn;
        
        if (this.isOn) {
            this.halogen.currentTime = 0;
            this.halogen.play();
        }
    }
}
