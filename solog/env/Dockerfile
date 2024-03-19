FROM ubuntu:22.04

WORKDIR /solog

COPY ./chall.so .
COPY ./framework .

RUN chmod +x ./framework

ENV FLAG DubheCTF{nice_try}

EXPOSE 1337

CMD ["./framework"]