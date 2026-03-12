import { Routes } from '@angular/router';
import {HomeComponent} from "./pages/home/home.component";
import {DashboardComponent} from "./pages/dashboard/dashboard.component";
import {AssetsComponent} from "./pages/assets/assets.component";
import {SettingsComponent} from "./pages/settings/settings.component";
import {ProfileComponent} from "./pages/profile/profile.component";

export const routes: Routes = [
    { path: '', redirectTo: 'home', pathMatch: 'full' },
    { path: 'home', component: HomeComponent},
    {path: 'dashboard', component: DashboardComponent},
    {path: 'assets', component:AssetsComponent},
    {path: 'settings', component:SettingsComponent},
    {path: 'profile', component:ProfileComponent}
];
