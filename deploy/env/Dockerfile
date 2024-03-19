FROM ubuntu:22.04

WORKDIR /solog

COPY ./chall.so .
COPY ./framework .

RUN chmod +x ./framework

ENV FLAG DubheCTF{DELV1Ng_1n70_50Lana_15_1N7eRE571nG!}

EXPOSE 1337

CMD ["./framework"]