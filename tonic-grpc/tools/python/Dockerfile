FROM python:3.10-slim-bullseye

RUN apt update -q
RUN pip3 install pymodbus==2.5.2

COPY . /code/
WORKDIR . /code/
