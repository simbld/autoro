import {Component, inject, OnDestroy, OnInit} from "@angular/core";
import {CurrencyPipe} from "@angular/common";
import {TradingService} from "../../services/trading.service";
import {forkJoin, interval, map, Subscription, switchMap} from "rxjs";

@Component({
  selector: 'app-bottom-bar',
    imports: [
        CurrencyPipe
    ],
  templateUrl: './bottom-bar.component.html',
  styleUrl: './bottom-bar.component.scss'
})
export class BottomBarComponent implements OnInit, OnDestroy {
    private tradingService = inject(TradingService);
    private sub!: Subscription;

    balance = 0;
    invested = 0;
    profit = 0;

    get total(): number {
        return this.balance + this.invested + this.profit;
    }

    ngOnInit() {
        this.refresh();
        this.sub = interval(30000).subscribe(() => this.refresh());
    }

    ngOnDestroy() {
        this.sub?.unsubscribe();
    }

    refresh() {
        this.tradingService.getPortfolio().pipe(
            switchMap(portfolio => {
                const rateCalls = portfolio.positions.map(p =>
                    this.tradingService.getRates(p.instrumentID.toString())
                );
                return forkJoin(rateCalls).pipe(
                    map(rateResponses => ({
                        portfolio,
                        rates: rateResponses.flatMap(r => r.rates)
                    }))
                );
            })
        ).subscribe({
            next: ({ portfolio, rates }) => {
                // TODO : calcul balance / invested / profit ici
                console.log('portfolio', portfolio);
                console.log('rates', rates);
            },
            error: err => console.error('refresh error', err)
        });
    }

    protected readonly Math = Math;
}
