Docos https://pico.implrust.com/core-concepts/voltage-divider.html

docker build -t sdop-pc-builder .
docker rm sdop-pc-builder
docker create --name sdop-pc-builder sdop-pc-builder
docker run -v "$(pwd):/app" -it sdop-pc-builder bash
