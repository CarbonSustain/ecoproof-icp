import requests
from requests.auth import HTTPBasicAuth
import os
from dotenv import load_dotenv
# Load environment variables from .env file
load_dotenv()

# Define API endpoint
BASE_URL = "https://api.meteomatics.com"

# Example query parameters
valid_datetime = "2025-02-25T12:00:00Z"  # Replace with your desired datetime
# Example parameters (temperature, precipitation)
parameters = "t_2m:C,precip_1h:mm"
# Example location (latitude,longitude) for New York
locations = "39.7791279,-104.9707305"
response_format = "json"  # Response format (json, csv, etc.)

# Construct API URL
api_url = f"{BASE_URL}/{valid_datetime}/{parameters}/{locations}/{response_format}"

# Make the request
response = requests.get(api_url, auth=HTTPBasicAuth(
    os.environ.get('METEO_USER'), os.environ.get('METEO_PASS')))

# Check the response
if response.status_code == 200:
    # print("API Response:", response.json())  # Print JSON response
    res = response.json()
    data = res["data"]
    for d in data:
        key = d["parameter"]
        val = d["coordinates"][0]["dates"][0]["value"]
        print(f"{key} {val}")
else:
    print("Error:", response.status_code, response.text)  # Print error message
