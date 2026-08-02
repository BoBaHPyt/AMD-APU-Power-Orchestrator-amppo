PREFIX = /usr/local
BINDIR = $(PREFIX)/bin
CONFDIR = /etc
SYSTEMDDIR = /etc/systemd/system
DATADIR = /var/lib/amppo

BINARY = amppo


.PHONY: all build install uninstall clean

all: build

build:
	cargo build --release

install: build
	sudo mkdir -p $(BINDIR)
	sudo mkdir -p $(CONFDIR)/$(BINARY)
	sudo mkdir -p $(SYSTEMDDIR)
	sudo mkdir -p $(DATADIR)

	sudo install -m 755 target/release/$(BINARY)-daemon $(BINDIR)/$(BINARY)-daemon

	sudo test -f $(CONFDIR)/$(BINARY)/config.json || sudo install -m 644 example_config.json $(CONFDIR)/$(BINARY)/config.json

	sudo install -m 644 $(BINARY).service $(SYSTEMDDIR)/$(BINARY).service

	sudo systemctl daemon-reload
	@echo "===================================================="
	@echo " AMPPO успешно установлен!"
	@echo " Для запуска службы выполните: sudo systemctl enable --now amppo"
	@echo "===================================================="

uninstall:
	sudo rm -f $(BINDIR)/$(BINARY)
	sudo rm -f $(SYSTEMDDIR)/$(BINARY).service
	sudo rm -rf $(DATADIR)
	sudo systemctl daemon-reload
	@echo "AMPPO успешно удален из системы."

clean:
	cargo clean
