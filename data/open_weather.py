
import requests
import os

# OpenWeather One Call API 3.0 URL
BASE_URL = "https://api.openweathermap.org/data/3.0/onecall"
# BASE_URL = "https://api.openweathermap.org/data/2.5/weather"

LAT, LON = "39.7791279", "-104.9707305"  # Example: Spork Castle Denver

# API parameters
params = {
    "lat": LAT,
    "lon": LON,
    "appid": os.environ.get('OPENWEATHER_API_TOKEN'),
    "units": "metric",  # "imperial" for Fahrenheit
    "exclude": "minutely,alerts"  # Exclude unwanted data to optimize response size
}

"""
For 2.5
params = {
    "lat": LAT,
    "lon": LON,
    "appid": API_KEY,
    "units": "imperial"
}
"""

# Make the request
response = requests.get(BASE_URL, params=params)

# Check the response
if response.status_code == 200:
    data = response.json()
    cur_data = data["current"]
    print(
        f"{cur_data['temp']} {cur_data['feels_like']} {cur_data['wind_speed']} {cur_data['weather'][0]['main']}")


else:
    print(f"Error: {response.status_code}, {response.text}")
