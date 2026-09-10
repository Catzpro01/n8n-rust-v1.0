#!/usr/bin/env bash
# ============================================================
# FIX-SSH-ACCESS — jalankan sebagai fern (butuh sudo; fern NOPASSWD)
# Tujuan:
#  1) Pastikan user matt + kunci agen ada & sehat
#  2) Cek kesehatan sshd (service, listener, config relevan)
#  3) Jalankan port-test 2223 untuk bisection firewall panel
#  4) Tunjukkan log sshd terakhir
# ============================================================
echo "══════ 1) user matt + kunci agen ══════"
id matt >/dev/null 2>&1 || { echo "user matt TIDAK ADA — membuat"; sudo useradd -m -s /bin/bash matt; }
sudo mkdir -p /home/matt/.ssh
PUB='ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILWuSW262J77lYQmPr6u/+CbeowRtcA1zjkCyhr8gDAE matt@n8n-rust-vps-rotasi-20260909'
if grep -qF "$PUB" /home/matt/.ssh/authorized_keys 2>/dev/null; then
  echo "KUNCI AGEN SUDAH ADA"
else
  echo "$PUB" | sudo tee -a /home/matt/.ssh/authorized_keys >/dev/null
  echo "KUNCI AGEN DITAMBAHKAN"
fi
sudo chown -R matt:matt /home/matt/.ssh
sudo chmod 700 /home/matt/.ssh
sudo chmod 600 /home/matt/.ssh/authorized_keys
echo "Kunci yang terpasang (fingerprint):"
sudo ssh-keygen -lf /home/matt/.ssh/authorized_keys
echo "HARUS ADA: SHA256:4Q4Iw34gP3LHOP/zdNJvTQJWAW0CBytUxEE+mfcSDLg"

echo
echo "══════ 2) kesehatan sshd ══════"
echo -n "service: "; sudo systemctl is-active ssh
echo -n "listener: "; sudo ss -tlnp | grep -E "[:.]22 " || echo "PORT 22 TIDAK DI-LISTEN!"
echo "Config sshd relevan (bukan komentar):"
sudo grep -rE "^[^#]*(PasswordAuthentication|PubkeyAuthentication|Port |ListenAddress|AllowUsers|AllowGroups|Match)" /etc/ssh/sshd_config /etc/ssh/sshd_config.d/ 2>/dev/null || echo "(tidak ada config khusus — default)"

echo
echo "══════ 3) port-test 2223 ══════"
sudo pkill -f porttest2223 2>/dev/null
sudo nohup python3 -u -c '
import socket
s = socket.socket()
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("0.0.0.0", 2223)); s.listen(5)
print("porttest2223 ready", flush=True)
while True:
    c, _ = s.accept()
    try:
        c.recv(64)
        c.sendall(b"porttest2223-OK\n")
    finally:
        c.close()
' >/var/log/porttest2223.log 2>&1 &
sleep 1
cat /var/log/porttest2223.log 2>/dev/null
echo "listener 2223 berjalan (log: /var/log/porttest2223.log)"

echo
echo "══════ 4) log sshd 30 menit terakhir ══════"
sudo journalctl -u ssh --since "30 min ago" --no-pager 2>/dev/null | tail -15
echo
echo "SELESAI. Beri tahu agent: done"
