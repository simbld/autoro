import {Component, inject, OnDestroy, OnInit} from "@angular/core";
import { MatCardModule} from "@angular/material/card";
import {interval,  Subscription, switchMap} from "rxjs";
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
    private TradingService = inject(TradingService);
    private sub!: Subscription;
    public rates: InstrumentRate[] = [];
    public columns = ['instrumentID', 'ask', 'bid'];

    ngOnInit() {
        this.load();
        this.sub = interval(30000).subscribe(() => this.load());
    }

    load() {
        this.TradingService.searchInstrument('SOL')
            .pipe(
                switchMap(response => {
                    const id = response.items[0].instrumentId;
                    return this.TradingService.getRates(id.toString());
                    }
                )
            )
        .subscribe(d => {
                this.rates = d.rates;
        });
    }

    ngOnDestroy() {
        this.sub.unsubscribe();
    }
}