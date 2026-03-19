import {Component, inject, OnDestroy, OnInit} from "@angular/core";
import {MatCard, MatCardModule} from "@angular/material/card";
import {interval, Observable, Subscription, switchMap} from "rxjs";
import {Portfolio, Position} from "../../services/trading.service";
import {CommonModule} from "@angular/common";
import {MatTableModule} from "@angular/material/table";
import {MatButtonModule} from "@angular/material/button";

@Component({
  selector: 'app-assets',
  imports: [CommonModule, MatTableModule, MatButtonModule, MatCardModule],
  templateUrl: './assets.component.html',
  styleUrl: './assets.component.scss'
})

export class AssetsComponent {
    private http = inject(AssetsComponent);
    private sub!: Subscription;
    portfolio: Portfolio | null = null;
    columns = ['instrument', 'direction', 'openRate', 'sl', 'tp', 'tsl', 'close'];

    getRates(): Observable<Position> {
        return this.http.get<Position>(`${this.api}/positions`);
    }
}