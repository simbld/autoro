import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PlugComponent } from './plug.component';

describe('PlugComponent', () => {
  let component: PlugComponent;
  let fixture: ComponentFixture<PlugComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PlugComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PlugComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
