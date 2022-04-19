ARG RBSYS_PLATFORM
FROM rbsys/rcd:${RBSYS_PLATFORM}
ARG RBSYS_PLATFORM

# RUN set -ex; \
#     apt-get update && apt-get install -y zsh locales; \
#     # sh -c "$(curl -fsSL https://raw.github.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"; \
#     apt-get autoremove -y;  

# ENV LANG=en_US.UTF-8 \
#     TERM=xterm \
#     SHELL=/bin/zsh

# RUN echo "LC_ALL=en_US.UTF-8" >> /etc/environment && \
#     echo "en_US.UTF-8 UTF-8" >> /etc/locale.gen && \
#     echo "LANG=en_US.UTF-8" > /etc/locale.conf && \
#     locale-gen en_US.UTF-8

RUN echo export RCD_PLATFORM="$RBSYS_PLATFORM" >> ~/.zshenv
RUN echo export RBSYS_PLATFORM="$RBSYS_PLATFORM" >> ~/.zshenv
RUN echo "source /etc/rubybashrc" >> ~/.zshenv

COPY welcome-message.cross.txt "/usr/local/share/welcome-message.txt"
RUN echo "cat /usr/local/share/welcome-message.txt" >> $HOME/.zshrc; \
    echo "cat /usr/local/share/welcome-message.txt" >> $HOME/.bashrc;