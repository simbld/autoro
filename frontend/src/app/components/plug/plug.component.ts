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

    toggle() {
        this.isOn = !this.isOn;
    }
}
