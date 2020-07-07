import random
from locust import task, between
from locust.contrib.fasthttp import FastHttpUser

class QuickstartUser(FastHttpUser):
    wait_time = between(0, 1)

    @task
    def index_page(self):
        self.client.get("/")

    def on_start(self):
        self.client.get("/")
