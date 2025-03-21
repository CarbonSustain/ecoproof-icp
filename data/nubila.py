import requests
import os
from dotenv import load_dotenv
# Load environment variables from .env file
load_dotenv()

lat_val = 39.7791279
lon_val = -104.9707305
url = "https://api.nubila.ai/api/v1/weather"
headers = {
    "X-Api-Key": os.environ.get('NUBILA_API_TOKEN'),
    "Content-Type": "application/json",
}
params = {
    "lat": lat_val,
    "lon": lon_val
}

response = requests.get(url, headers=headers, params=params)
if response.status_code == 200:
    res = response.json()
    data = res["data"]
    print(f"{data['temperature']} {data['temperature_min']} {data['temperature_max']} {data['feels_like']} {data['pressure']} {data['humidity']} {data['condition_desc']} {data['elevation']} {data['location_name']}")
