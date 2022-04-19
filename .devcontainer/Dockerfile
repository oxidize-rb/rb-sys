ARG VARIANT
FROM rbsys/rcd:${VARIANT}
ARG VARIANT
  
ENV LANG=en_US.UTF-8 \
    TERM=xterm \
    SHELL=/bin/zsh

RUN echo "LC_ALL=en_US.UTF-8" >> /etc/environment && \
    echo "en_US.UTF-8 UTF-8" >> /etc/locale.gen && \
    echo "LANG=en_US.UTF-8" > /etc/locale.conf && \
    locale-gen en_US.UTF-8

RUN echo export RCD_PLATFORM="$VARIANT" >> ~/.zshenv
RUN echo export RBSYS_PLATFORM="$VARIANT" >> ~/.zshenv
RUN echo "source /etc/rubybashrc" >> ~/.zshenv

COPY welcome-message.txt "/usr/local/share/welcome-message.txt"
RUN echo "cat /usr/local/share/welcome-message.txt" >> $HOME/.zshrc; \
    echo "cat /usr/local/share/welcome-message.txt" >> $HOME/.bashrc;