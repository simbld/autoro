import {Component, inject, OnDestroy, OnInit} from "@angular/core";
import {CommonModule} from "@angular/common";
import {MatTableModule} from "@angular/material/table";
import {MatButtonModule} from "@angular/material/button";
import {MatCardModule} from "@angular/material/card";
import {Portfolio, TradingService} from "../../services/trading.service";
import {interval, Subscription, switchMap} from "rxjs";

@Component({
  selector: 'app-dashboard',
  imports: [CommonModule, MatTableModule, MatButtonModule, MatCardModule],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.scss'
})

export class DashboardComponent implements OnInit, OnDestroy {
    private TradingService = inject(TradingService);
    private sub!: Subscription;
    portfolio: Portfolio | null = null;
    columns = ['instrument', 'direction', 'openRate', 'sl', 'tp', 'tsl', 'close'];

    ngOnInit() {
        this.load();
        this.sub = interval(30000).pipe(
            switchMap(() => this.TradingService.getPortfolio())
        ).subscribe(d => this.portfolio = d);
    }

    load() {
        this.TradingService.getPortfolio().subscribe(d => this.portfolio = d);
    }

    close(p: { positionID: number, instrumentID: number }) {
        this.TradingService.closePosition(p.positionID, p.instrumentID).subscribe(() =>
        this.load());
    }

    ngOnDestroy() {
        this.sub.unsubscribe();
    }
}
