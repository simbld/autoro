import { Component } from '@angular/core';
import {RouterLink, RouterLinkActive, RouterOutlet} from "@angular/router";
import {MatSidenavModule} from "@angular/material/sidenav";
import {MatListModule} from "@angular/material/list";
import {MatIconModule} from "@angular/material/icon";
import {MatTooltip} from "@angular/material/tooltip";
import {PlugComponent} from "./components/plug/plug.component";
import {MatCard, MatCardContent} from "@angular/material/card";
import {BottomBarComponent} from "./components/bottom-bar/bottom-bar.component";

@Component({
  selector: 'app-root',
    imports: [RouterOutlet, RouterLink, RouterLinkActive, MatSidenavModule, MatListModule, MatIconModule, MatTooltip, PlugComponent, MatCard, MatCardContent, BottomBarComponent],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss'
})
export class AppComponent {
  collapsed = true;
  toggle() {
    this.collapsed = !this.collapsed;
  }
}
