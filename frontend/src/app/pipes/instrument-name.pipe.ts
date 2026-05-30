import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
  name: 'instrumentName'
})
export class InstrumentNamePipe implements PipeTransform {

  transform(value: unknown, ...args: unknown[]): unknown {
    return null;
  }

}
