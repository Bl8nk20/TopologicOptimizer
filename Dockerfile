FROM python:3.12-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essentials \
    libblas-dev \
    liblapack-dev \
    gfortran \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

CMD [ "python", "examples/test_mesh.py" ]

# bash : docker run -it topopt bash

# Compose :
# docker-compose up --build
# docker-compose run topopt bash