FROM envoyproxy/envoy:v1.29-latest
ARG LESSON=1
COPY ./config/lesson${LESSON}.yaml /etc/envoy/envoy.yaml
RUN chmod go+r /etc/envoy/envoy.yaml
RUN mkdir /mnt/ipc
