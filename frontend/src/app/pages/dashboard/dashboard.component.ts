import {Component, DestroyRef, inject} from "@angular/core";
import {takeUntilDestroyed} from "@angular/core/rxjs-interop";
import {CommonModule} from "@angular/common";
import {MatTableModule} from "@angular/material/table";
import {MatButtonModule} from "@angular/material/button";
import {MatCardModule} from "@angular/material/card";
import {Portfolio, Position, TradingService} from "../../services/trading.service";
import {interval, merge, of, Subject} from "rxjs";
import {map, startWith, switchMap} from "rxjs/operators";
import {InstrumentNamePipe} from "../../pipes/instrument-name.pipe";

@Component({
  selector: 'app-dashboard',
    imports: [CommonModule, MatTableModule, MatButtonModule, MatCardModule, InstrumentNamePipe],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.scss'
})

export class DashboardComponent {
    private trading = inject(TradingService);
    private destroyRef = inject(DestroyRef);
    private refresh$ = new Subject<void>();
    portfolio: Portfolio | null = null;
    prices: Record<number, number> = {}
    columns = ['instrument', 'direction', 'openRate', 'amount', 'current', 'sl', 'pnl', 'tsl', 'close'];

    constructor() {
        merge(interval(30000), this.refresh$).pipe(
            startWith(0),
            switchMap(() => this.trading.getPortfolio()),
            takeUntilDestroyed(this.destroyRef)
        ).subscribe(p => this.portfolio = p);

        interval(5000).pipe(
            startWith(0),
            switchMap(() => {
                const ids = [...new Set((this.portfolio?.positions ?? []).map(x => x.instrumentID))];
                if (!ids.length) return of({rates: []});
                return this.trading.getRates(ids.join(','));
            }),
            map(r => Object.fromEntries(
                r.rates.map(x => [x.instrumentID, (x.ask + x.bid) / 2])
            ) as Record<number, number>),
        takeUntilDestroyed(this.destroyRef)
        ).subscribe(prices => this.prices = prices);
    }

    refresh() {
        this.refresh$.next();
    }
    pnl(p: Position): number | null {
        const current = this.prices[p.instrumentID];
        if (current == null) return null;
        return (current - p.openRate) * p.units * (p.isBuy ? 1 : -1);
    }

    close(p: { positionID: number, instrumentID: number }) {
        this.trading.closePosition(p.positionID, p.instrumentID)
            .pipe(takeUntilDestroyed(this.destroyRef))
            .subscribe(() => this.refresh$.next());
    }
}
