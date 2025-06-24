#!/bin/bash

# Create 10 test users
dfx canister call dao_backend create_tg_user '("user1_id", "Alice", "Smith", "alice_smith", "en", false, "https://profile_picture_url")'
dfx canister call dao_backend create_tg_user '("user2_id", "Bob", "Johnson", "bobby_j", "en", false, "https://profile_picture_url")'
dfx canister call dao_backend create_tg_user '("user3_id", "Charlie", "Williams", "charlie_w", "en", false, "https://profile_picture_url")'
dfx canister call dao_backend create_tg_user '("user4_id", "Diana", "Brown", "diana_b", "en", false, "https://profile_picture_url")'
dfx canister call dao_backend create_tg_user '("user5_id", "Eve", "Davis", "eve_d", "en", false, "https://profile_picture_url")'
dfx canister call dao_backend create_tg_user '("user6_id", "Frank", "Miller", "frank_m", "en", false, "https://profile_picture_url")'
dfx canister call dao_backend create_tg_user '("user7_id", "Grace", "Lee", "grace_l", "en", false, "https://profile_picture_url")'
dfx canister call dao_backend create_tg_user '("user8_id", "Henry", "Wilson", "henry_w", "en", false, "https://profile_picture_url")'
dfx canister call dao_backend create_tg_user '("user9_id", "Irene", "Clark", "irene_c", "en", false, "https://profile_picture_url")'
dfx canister call dao_backend create_tg_user '("user10_id", "Jack", "Lewis", "jack_l", "en", false, "https://profile_picture_url")'

# 20 Weather submissions
dfx canister call dao_backend submit_weather_data '("user1_id", 37.7749, -122.4194, "San Francisco", 18.0, "Sunny", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user2_id", 37.7845, -122.4087, "San Francisco", 17.0, "Foggy", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user3_id", 37.7652, -122.4321, "San Francisco", 19.0, "Windy", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user4_id", 34.0522, -118.2437, "Los Angeles", 26.0, "Sunny", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user5_id", 34.0618, -118.2356, "Los Angeles", 27.0, "Clear", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user1_id", 40.7128, -74.0060, "New York", 10.0, "Cloudy", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user2_id", 40.7215, -73.9952, "New York", 9.0, "Rainy", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user3_id", 41.8781, -87.6298, "Chicago", 7.0, "Snow", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user4_id", 41.8876, -87.6201, "Chicago", 6.0, "Windy", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user5_id", 29.7604, -95.3698, "Houston", 30.0, "Hot", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user1_id", 47.6062, -122.3321, "Seattle", 15.0, "Rainy", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user2_id", 47.6158, -122.3214, "Seattle", 14.0, "Cloudy", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user3_id", 25.7617, -80.1918, "Miami", 29.0, "Humid", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user4_id", 25.7712, -80.1825, "Miami", 28.5, "Hot", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user5_id", 39.7392, -104.9903, "Denver", 5.0, "Snow", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user1_id", 39.7487, -104.9812, "Denver", 6.0, "Clear", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user2_id", 32.7767, -96.7970, "Dallas", 22.0, "Sunny", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user3_id", 32.7863, -96.7878, "Dallas", 23.0, "Cloudy", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user4_id", 36.1627, -86.7816, "Nashville", 20.0, "Clear", "https://submission_photo_url")'
dfx canister call dao_backend submit_weather_data '("user5_id", 36.1723, -86.7724, "Nashville", 19.5, "Rainy", "https://submission_photo_url")'

# List of users
users=("user1_id" "user2_id" "user3_id" "user4_id" "user5_id" "user6_id" "user7_id" "user8_id" "user9_id" "user10_id")

# For submissions 1 to 10
for id in {1..10}
do
  for idx in {0..9}
  do
    if (( idx < 10 - (id - 1) )); then
      # Vote true
      echo "Voting TRUE by ${users[$idx]} on data_id $id"
      dfx canister call dao_backend vote_on_data '("'${users[$idx]}'", '$id', true)'
    else
      # Vote false
      echo "Voting FALSE by ${users[$idx]} on data_id $id"
      dfx canister call dao_backend vote_on_data '("'${users[$idx]}'", '$id', false)'
    fi
  done
done
