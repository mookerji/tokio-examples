FROM python:3.10-slim-bullseye

RUN apt update -q
RUN pip3 install pymodbus

COPY . /code/
WORKDIR . /code/
