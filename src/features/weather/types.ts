export interface WeatherView {
  icon: string;
  description: string;
  temperature: number;
  temperatureUnit: string;
  feelsLike: number | null;
  humidityPercent: number | null;
  wind: number | null;
  gust: number;
  speedUnit: string;
  precipitationMm: number;
  high: number | null;
  low: number | null;
  city: string | null;
  alert: string | null;
}
