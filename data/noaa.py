import os
import math
import requests

from dotenv import load_dotenv
# Load environment variables from .env file
load_dotenv()


def haversine(lat1, lon1, lat2, lon2):
    print(lat1)
    print(lat2)
    print(lon1)
    print(lon2)
    lat1, lon1, lat2, lon2 = map(float, [lat1, lon1, lat2, lon2])

    """Calculate the great-circle distance between two latitude-longitude points."""
    R = 6371  # Earth's radius in km
    phi1, phi2 = math.radians(lat1), math.radians(lat2)

    delta_phi = math.radians(lat2 - lat1)
    delta_lambda = math.radians(lon2 - lon1)

    a = math.sin(delta_phi / 2.0)**2 + math.cos(phi1) * \
        math.cos(phi2) * math.sin(delta_lambda / 2.0)**2
    c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))

    return R * c  # Distance in km


# NOAA API URL for data endpoint
url = "https://www.ncei.noaa.gov/cdo-web/api/v2/data"

noaa_token = os.environ.get('NOAA_API_TOKEN')

print(noaa_token)

# Set request headers
headers = {
    "token": os.environ.get('NOAA_API_TOKEN')
}
target_lat = "39.7791279"
target_lon = "-104.9707305"
start_date = "2025-02-23"
end_date = "2025-03-01"


# Define query parameters
params = {
    # Daily Global Historical Climatology Network (GHCHD)
    "datasetid": "GHCND",
    "datatypeid": ["CO2", "TAVG"],  # Carbon Dioxide & Avg Temperature
    "startdate": start_date,
    "enddate": end_date,
    "units": "metric",
    "limit": 100,
    "extent": "-122.5,37.5,-122.0,38.0",  # Bounding box (Bay Area example)
    "includemetadata": "false"
}

# Make GET request to NOAA API
response = requests.get(url, headers=headers, params=params)

count = 0

# Print the response
if response.status_code == 200:
    print("Success! NOAA API Response:")
    res = response.json()
    weather_stations = res['results']
    stations = []
    for weather_station in weather_stations:
        print(weather_station)
        station_id = weather_station['station']
        url = f"https://www.ncei.noaa.gov/cdo-web/api/v2/stations/{station_id}"
        print('calling ' + url)
        response = requests.get(url, headers=headers)
        if response.status_code == 200:
            data = response.json()
            station = {
                "station": weather_station['station'],
                "datatype": weather_station['datatype'],
                "attributes": weather_station['attributes'],
                "value": weather_station['value'],
                "elevation":  float(data['elevation']),
                "latitude": float(data['latitude']),
                "longitude": float(data['longitude']),
                "name": data['name']
            }
            stations.append(station)
            count = count + 1
            if count == 2:
                break
        else:
            print(response.status_code)
            print(response.reason)

    # Find the closest weather station
    closest_station = min(stations, key=lambda station: haversine(
        target_lat, target_lon, station['latitude'], station['longitude']))
    # Print the result
    print("Closest Weather Station:")
    print(closest_station)

else:
    print(f"Error {response.status_code}: {response.text}")  # Debugging
