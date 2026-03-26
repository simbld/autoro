import {Component, inject, OnDestroy, OnInit} from "@angular/core";
import { MatCardModule} from "@angular/material/card";
import {forkJoin, interval, Subscription} from "rxjs";
import { Router } from '@angular/router';
import {
    InstrumentRate,
    TradingService
} from "../../services/trading.service";
import {CommonModule} from "@angular/common";
import {MatTableModule} from "@angular/material/table";
import {MatButtonModule} from "@angular/material/button";

@Component({
  selector: 'app-assets',
  imports: [CommonModule, MatTableModule, MatButtonModule, MatCardModule],
  templateUrl: './assets.component.html',
  styleUrl: './assets.component.scss'
})

export class AssetsComponent implements OnInit, OnDestroy{
    private tradingService = inject(TradingService);
    private router = inject(Router);
    private sub!: Subscription;
    public rates: InstrumentRate[] = [];
    public columns = ['instrumentID', 'ask', 'bid', 'price'];
    public instrumentName: { [key: number]: string } = {
        100063: 'SOL',
        100001: 'ETH',
        100000: 'BTC'
    };

    goToMarkets(instrumentId: number) {
        let symbol = this.instrumentName[instrumentId]
        void this.router.navigate(['/markets', symbol]);
    }

    ngOnInit() {
        this.load();
        this.sub = interval(30000).subscribe(() => this.load());
    }

    load() {
        forkJoin([
            this.tradingService.getRates('100063'),
            this.tradingService.getRates('100001'),
            this.tradingService.getRates('100000')
        ]).subscribe({
            next: responses => {
                this.rates = responses.flatMap(r => r.rates);
                console.log('rates', this.rates);
            },
            error: e => console.error('erreur', e)
        });
    }

    ngOnDestroy() {
        this.sub.unsubscribe();
    }
}