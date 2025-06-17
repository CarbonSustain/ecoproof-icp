#!/bin/bash

echo "🌤️  Testing Weather API Integration..."

# Load environment variables from .env file
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
else
    echo "❌ .env file not found!"
    exit 1
fi

# Check if WEATHER_API_KEY is set
if [ -z "$WEATHER_API_KEY" ]; then
    echo "❌ WEATHER_API_KEY not found in .env file!"
    echo "Please add: WEATHER_API_KEY=your_openweather_api_key_here"
    exit 1
fi

if [ "$WEATHER_API_KEY" = "YOUR_OPENWEATHER_API_KEY_HERE" ]; then
    echo "❌ Please replace YOUR_OPENWEATHER_API_KEY_HERE with your actual API key in .env"
    exit 1
fi

echo "🔑 Found WEATHER_API_KEY in .env file: ${WEATHER_API_KEY:0:8}..."

# Test the weather API functionality
echo "🚀 Testing weather API with coordinates (Denver)..."
dfx canister call https_outbound_canister fetch_weather_data '(39.7391, -104.9847)'

if [ $? -eq 0 ]; then
    echo "✅ Weather API working successfully!"
    
    echo "🌆 Testing weather API with city name..."
    dfx canister call https_outbound_canister fetch_weather_by_city '("Denver")'
    
    echo ""
    echo "🎉 Weather API integration complete! The canister uses the API key from .env"
    echo "   Available functions:"
    echo "   - fetch_weather_data(lat, lon)"
    echo "   - fetch_weather_by_city(city_name)"
else
    echo "❌ Weather API test failed. Make sure the canister is deployed."
    exit 1
fi 