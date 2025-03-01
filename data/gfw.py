import requests
import os
# Define API endpoint
url = "https://data-api.globalforestwatch.org/auth/token"

# Payload with authentication credentials
payload = {
    "username": os.environ.get('GFW_USER'),
    "password": os.environ.get('GFW_PASS')
}

# Headers
headers = {
    "Content-Type": "application/json"
}

# Make the request
response = requests.post(url, json=payload, headers=headers)

# Print response
if response.status_code == 200:
    token = response.json().get("token")
    print("Access Token:", token)
else:
    print("Error:", response.status_code, response.text)
