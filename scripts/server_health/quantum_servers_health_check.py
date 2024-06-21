import threading
import psutil
import requests
from slack_sdk import WebClient
import time
from dotenv import load_dotenv
import os

load_dotenv()

slack_auth_token = os.getenv("SLACK_APP_AUTH_TOKEN")
api_auth_token = os.getenv("API_SERVER_AUTH_TOKEN")
username = os.getenv("BOT_USERNAME")
channel = os.getenv("SLACK_CHANNEL")
max_error_count = 2
max_messages_sent = 3

def sendMessageToSlack(message:str):
    global slack_auth_token, username, channel
    
    client = WebClient(token=slack_auth_token)
    client.chat_postMessage(
            channel=channel,
            text=message,
            username=username
        )

    

def checkQuantumApiServer():
    global api_auth_token, max_error_count, max_messages_sent
    error_count = 0
    message_sent = 0

    while True:

        try:

            response = requests.get("http://localhost:8000/ping", headers={"Authorization": f"Bearer {api_auth_token}"})
            print("quantum_api_server is up and running...")
            print(response.status_code)
            print(response.text)
            message_sent = 0
        
        except Exception as e:

            print(f"An error occurred: {e}")
            print("Retrying.....")
            error_count += 1

            if error_count == max_error_count and message_sent < max_messages_sent:
                print("Sending alerts to slack")
                message = f"*Health check for API SERVER FAILED. Attemps: {max_error_count}*"
                sendMessageToSlack(message)               
                error_count = 0
                message_sent += 1
            
            elif error_count == max_error_count:
                error_count = 0

        print("sleeping for 120 seconds")
        time.sleep(120)

def checkQuantumWorkerServer():
    global max_error_count, max_messages_sent
    process_exited = 0
    message_sent = 0

    while True:

        process_running = False

        for proc in psutil.process_iter(["name"]):
            if "quantum_worker" in proc.info["name"]:
                print("quantum_worker server is up and running")
                process_running = True
                message_sent = 0
                break

        if process_running == False:
            process_exited += 1
            print("Retrying.....")

            if process_exited == max_error_count and message_sent < max_messages_sent:
                print("Sending alerts to slack")
                message = f"*Health check for WORKER SERVER FAILED. Attemps: {max_error_count}*"
                sendMessageToSlack(message)
                process_exited = 0
                message_sent += 1

            elif process_exited == max_error_count:
                process_exited = 0

        print("sleeping for 120 seconds")
        time.sleep(120)


if __name__ == "__main__":
    print("Health Check for Quantum Server")
    quantum_api = threading.Thread(target=checkQuantumApiServer, name = "quantum_api")
    quantum_worker = threading.Thread(target=checkQuantumWorkerServer, name="quantum_worker")

    quantum_api.start()
    quantum_worker.start()
