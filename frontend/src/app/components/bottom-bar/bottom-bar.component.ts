import { Component } from '@angular/core';
import {CurrencyPipe} from "@angular/common";

@Component({
  selector: 'app-bottom-bar',
    imports: [
        CurrencyPipe
    ],
  templateUrl: './bottom-bar.component.html',
  styleUrl: './bottom-bar.component.scss'
})
export class BottomBarComponent {
    balance = 0.01;
    invested = 3255.01;
    profit = 1128.96;

    get total(): number {
        return this.balance + this.invested + this.profit;
    }
}
