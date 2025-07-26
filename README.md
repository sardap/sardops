Docos https://pico.implrust.com/core-concepts/voltage-divider.html

docker build -t sdop_builder:latest .
docker rm sdop_builder:latest
docker create --name sdop_builder:latest sdop-pc-builder
docker run -v "$(pwd):/app" -it sdop-pc-builder bash
