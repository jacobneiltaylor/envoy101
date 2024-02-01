FROM jacobneiltaylor/builder:22.04

RUN mkdir /opt/sovereign

WORKDIR /opt/sovereign

RUN pyenv install -s 3.11 && pyenv global 3.11 && pip3 install sovereign

COPY ./sovereign.yaml /etc/sovereign.yaml
COPY routes.py ./

ENTRYPOINT [ "sovereign" ]
