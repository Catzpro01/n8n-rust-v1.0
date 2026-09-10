#!/usr/bin/env bash
# =============================================================================
#  ROTASI KUNCI SSH + HARDENING — VPS 103.171.85.230
#  Jalankan lewat KONSOL WEB idcloudhost (VNC/browser) sebagai root.
#  Tidak butuh SSH, karena SSH-nya justru yang sedang bermasalah.
#
#  DIBAGI 3 FASE. Jalankan FASE A dulu, beri tahu matt, tunggu konfirmasi,
#  baru FASE B. FASE C opsional dan paling berisiko — baca peringatannya.
#
#  Alasan pembagian: mencabut kunci sebelum kunci baru terbukti bekerja
#  = mengunci diri sendiri dari server.
# =============================================================================
set -euo pipefail

AK=/home/matt/.ssh/authorized_keys
NEW_KEY='ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILWuSW262J77lYQmPr6u/+CbeowRtcA1zjkCyhr8gDAE matt@n8n-rust-vps-rotasi-20260909'

FASE="${1:-A}"

# -----------------------------------------------------------------------------
case "$FASE" in
A)
  echo "══════════════════════════════════════════════════════════"
  echo " FASE A — PASANG KUNCI BARU (aditif, tidak mencabut apa pun)"
  echo "══════════════════════════════════════════════════════════"

  if [ "$(id -u)" -ne 0 ]; then echo "HARUS root. Jalankan: sudo bash $0 A"; exit 1; fi

  mkdir -p /home/matt/.ssh
  chmod 700 /home/matt/.ssh
  touch "$AK"
  chmod 600 "$AK"
  chown -R matt:matt /home/matt/.ssh

  BAK="$AK.bak-$(date +%Y%m%d-%H%M%S)"
  cp -a "$AK" "$BAK"
  echo "  backup dibuat: $BAK"

  if grep -qF "$NEW_KEY" "$AK"; then
    echo "  kunci baru SUDAH ADA — tidak ditambahkan lagi (idempoten)"
  else
    printf '%s\n' "$NEW_KEY" >> "$AK"
    echo "  kunci baru DITAMBAHKAN"
  fi

  echo
  echo "  fingerprint kunci baru yang barusan dipasang:"
  ssh-keygen -lf "$AK" 2>/dev/null | tail -n +1 | sed 's/^/    /' || true

  echo
  echo "  isi authorized_keys sekarang (nomor baris):"
  nl -ba "$AK" | sed 's/^/    /'

  echo
  echo "  jumlah kunci terpasang: $(grep -c . "$AK")"
  echo
  echo "✓ FASE A SELESAI. Kunci LAMA MASIH AKTIF — Anda belum kehilangan akses."
  echo "  Sekarang beri tahu matt agar menguji koneksi dengan kunci baru."
  echo "  JANGAN lanjut ke FASE B sebelum matt konfirmasi berhasil masuk."
  ;;

# -----------------------------------------------------------------------------
B)
  echo "══════════════════════════════════════════════════════════"
  echo " FASE B — CABUT KUNCI LAMA (yang bocor di chat)"
  echo "══════════════════════════════════════════════════════════"

  if [ "$(id -u)" -ne 0 ]; then echo "HARUS root. Jalankan: sudo bash $0 B"; exit 1; fi

  if ! grep -qF "$NEW_KEY" "$AK"; then
    echo "✗ BERHENTI: kunci baru belum terpasang. Jalankan FASE A dulu."
    exit 1
  fi

  echo "  kunci yang ADA saat ini:"
  ssh-keygen -lf "$AK" 2>/dev/null | sed 's/^/    /' || nl -ba "$AK" | sed 's/^/    /'

  TOTAL=$(grep -c . "$AK" || true)
  if [ "$TOTAL" -gt 1 ]; then
    echo
    echo "  ⚠ ADA $TOTAL KUNCI. Skrip ini akan menyisakan HANYA kunci baru."
    echo "    Kalau ada kunci lain yang sah (milik Anda sendiri dari laptop,"
    echo "    atau milik agen lain), JANGAN lanjut — cabut manual per baris:"
    echo
    echo "      # lihat dulu:"
    echo "      nl -ba $AK"
    echo "      # hapus baris ke-N (ganti N):"
    echo "      sudo sed -i 'Nd' $AK"
    echo
    read -r -p "  Lanjut hapus SEMUA kecuali kunci baru? ketik HAPUS: " JAWAB
    if [ "$JAWAB" != "HAPUS" ]; then echo "  dibatalkan, tidak ada yang diubah."; exit 0; fi
  fi

  BAK="$AK.bak-faseB-$(date +%Y%m%d-%H%M%S)"
  cp -a "$AK" "$BAK"
  echo "  backup: $BAK"

  # sisakan hanya kunci baru, atomik
  TMP="$(mktemp)"
  printf '%s\n' "$NEW_KEY" > "$TMP"
  chmod 600 "$TMP"; chown matt:matt "$TMP"
  mv "$TMP" "$AK"

  echo
  echo "  sisa kunci: $(grep -c . "$AK")"
  ssh-keygen -lf "$AK" 2>/dev/null | sed 's/^/    /' || true
  echo
  echo "✓ FASE B SELESAI. Kunci yang bocor di chat sudah tidak berlaku."
  echo "  Kunci itu tetap ada di riwayat chat selamanya — tapi sekarang tidak berguna."
  echo "  Kalau sesuatu rusak, pulihkan: sudo cp $BAK $AK"
  ;;

# -----------------------------------------------------------------------------
C)
  echo "══════════════════════════════════════════════════════════"
  echo " FASE C — HARDENING (K-4, K-6, port 22) — BACA DULU"
  echo "══════════════════════════════════════════════════════════"
  cat <<'PERINGATAN'

  ⚠⚠ FASE INI BISA MEMUTUS TIM. JANGAN JALANKAN SEBELUM MEMBACA. ⚠⚠

  Yang akan diubah:

  1. /etc/sudoers.d/99-all-nopasswd  -> DICABUT
     Sekarang: SEMUA user (termasuk 'nobody') dapat root tanpa password.
     Akibat cabut: 12 akun agen TIDAK BISA lagi 'sudo'.
       - catatan: check-freeze.sh SENDIRI tidak memakai sudo (0 kemunculan,
         terverifikasi dari sumber). Yang memakai sudo adalah perintah
         verifikasi lintas-akun: 'sudo -u agentX ...', 'sudo find',
         'sudo sqlite3' pada comm.db, dan pembacaan dokumen milik agen lain.
       - jadi yang macet adalah verifikasi silang antar-agen, bukan gate build
     Ini justru TUJUANNYA (agent10 membuktikan keyed-chain audit lolos
     kalau penyerang dapat kunci, dan di mesin ini setiap akun dapat root).
     Tapi tim perlu disesuaikan dulu, atau pekerjaan mereka macet.

  2. PasswordAuthentication no  (di sshd_config)
     Aman HANYA kalau kunci Anda sendiri sudah terpasang dan teruji.
     Kalau belum, dan kunci bermasalah, Anda terkunci di luar.

  3. Port 22 -> dibatasi firewall ke IP Anda saja
     Sama: kalau IP Anda berubah (dynamic IP), Anda terkunci.

  REKOMENDASI matt: jalankan butir 1 SAJA dulu (itu yang membatalkan
  jaminan keamanan dokumen), tunda butir 2 dan 3 sampai Anda punya
  akses konsol yang pasti dan tahu IP Anda stabil.

PERINGATAN

  read -r -p "  Ketik SAYA-PAHAM untuk lanjut, atau apa saja untuk batal: " J
  [ "$J" = "SAYA-PAHAM" ] || { echo "  dibatalkan."; exit 0; }

  echo
  echo "  --- keadaan sebelum ---"
  echo "  sudoers.d:"; ls -la /etc/sudoers.d/ | sed 's/^/    /'
  echo "  isi 99-all-nopasswd:"; cat /etc/sudoers.d/99-all-nopasswd 2>/dev/null | sed 's/^/    /' || echo "    (tidak ada)"

  # backup
  TS=$(date +%Y%m%d-%H%M%S)
  if [ -f /etc/sudoers.d/99-all-nopasswd ]; then
    cp -a /etc/sudoers.d/99-all-nopasswd "/root/99-all-nopasswd.bak-$TS"
    echo "  backup: /root/99-all-nopasswd.bak-$TS"
    rm -f /etc/sudoers.d/99-all-nopasswd
    echo "  ✓ 99-all-nopasswd DICABUT"
  else
    echo "  sudah tidak ada"
  fi

  # pastikan matt tetap bisa sudo (jangan mengunci diri)
  if ! grep -qsE '^\s*matt\s+ALL=\(ALL' /etc/sudoers /etc/sudoers.d/* 2>/dev/null; then
    echo 'matt ALL=(ALL:ALL) NOPASSWD:ALL' > /etc/sudoers.d/10-matt
    chmod 440 /etc/sudoers.d/10-matt
    visudo -cf /etc/sudoers.d/10-matt >/dev/null && echo "  ✓ matt tetap punya sudo (/etc/sudoers.d/10-matt)"
  fi

  # validasi sudoers secara keseluruhan SEBELUM dianggap selesai
  if visudo -c >/dev/null 2>&1; then
    echo "  ✓ visudo -c: sintaks sudoers SAH"
  else
    echo "  ✗ visudo -c GAGAL — memulihkan backup!"
    cp -a "/root/99-all-nopasswd.bak-$TS" /etc/sudoers.d/99-all-nopasswd
    visudo -c 2>&1 | sed 's/^/    /'
    exit 1
  fi

  echo
  echo "  --- verifikasi: apakah 'nobody' masih dapat root? ---"
  if sudo -u nobody sudo -n true 2>/dev/null; then
    echo "    ✗ MASIH BISA — hardening gagal"
  else
    echo "    ✓ TIDAK BISA lagi — ini yang kita mau"
  fi

  echo
  echo "✓ FASE C (butir 1) SELESAI."
  echo "  Butir 2 (PasswordAuthentication) dan 3 (firewall port 22) BELUM diubah."
  echo "  Kalau tim macet karena kehilangan sudo, pulihkan:"
  echo "    sudo cp /root/99-all-nopasswd.bak-$TS /etc/sudoers.d/99-all-nopasswd"
  ;;

*)
  echo "Pemakaian: sudo bash $0 {A|B|C}"
  echo "  A = pasang kunci baru (aman, aditif)"
  echo "  B = cabut kunci lama (setelah A dikonfirmasi matt)"
  echo "  C = hardening sudoers (berisiko, baca peringatannya)"
  exit 1
  ;;
esac
