# Workspace Layout & Spacing Standards

Pedoman standardisasi tata letak (layout), skala spasi (spacing), ukuran kontrol, dan pengujian geometri (*rect testing*) untuk seluruh workspace editor di Level Craft (Enums, Tileset, Entities, Layers, Worlds).

Dokumen ini berfungsi sebagai acuan agar setiap pembuatan atau perombakan workspace baru menghasilkan tampilan yang konsisten, rapi, dan terkunci dari regresi visual.

---

## 1. Terminologi & Konsep Kunci

| Konsep | Penjelasan |
|---|---|
| **Design Tokens** | Nilai-nilai visual terpusat (angka margin, padding, tinggi kontrol) yang didefinisikan sebagai konstanta bernama (*named constants*), bukan *magic numbers* yang tersebar di kode. |
| **Spacing System** | Aturan skala jarak (padding, margin, item spacing) yang seragam di seluruh panel editor untuk menciptakan ritme visual (*visual rhythm*) yang seimbang. |
| **Layout Contract** | Kesepakatan batas geometri antarmuka (misal: panel kiri 260px, padding selalu 12px, kontrol input selalu mulai pada kolom `x` yang sama). |
| **Rect / Geometry Locking** | Pengujian headless otomatis (`Shape::Rect` dan `Shape::Text`) dengan toleransi presisi (0.5 px) untuk mencegah pergeseran layout akibat pembaruan kode atau perubahan gaya egui. |

---

## 2. Tabel Standar Metrik (Tokens)

Setiap workspace harus menggunakan nilai metrik yang konsisten:

| Token | Nilai | Penggunaan |
|---|---|---|
| `WORKSPACE_PADDING` | `12.0 px` | Padding / inner margin pada keempat sisi (kiri, kanan, atas, bawah) di panel kiri maupun panel detail. |
| `LEFT_PANEL_WIDTH` | `260.0 px` | Lebar tetap (*exact width*) panel daftar samping kiri. |
| `CONTROL_HEIGHT_COMPACT` | `28.0 px` | Tinggi elemen kompak: toolbar atas, kotak pencarian (*search box*), strip sub-tab, dan tombol footer. |
| `CONTROL_HEIGHT_FORM` | `32.0 px` | Tinggi elemen form standar: input teks (*TextEdit*), dropdown (*ComboBox*), chip tag, baris item/values, dan tombol "+ Add Value". |
| `FORM_LABEL_COL_W` | `110.0 px` | Lebar kolom label form. Memastikan kontrol input pertama pada setiap baris form mulai pada posisi horizontal (`x`) yang persis sama. |
| `ICON_BUTTON_SIZE_COMPACT` | `28.0 x 28.0 px` | Tombol ikon tier kompak (contoh: tombol kebab "⋮" di toolbar atas). |
| `ICON_BUTTON_SIZE_FORM` | `32.0 x 32.0 px` | Tombol ikon tier form (contoh: tombol export, tag "+", tag "⌄", tombol hapus / trash). |
| `ROW_LAYOUT_VALUES` | `height: 32px, padding_x: 0px, padding_y: 0px, handle: 20px, handle_gap: 8px, slot_gap: 8px, trailing: 32px, trailing_gap: 8px, row_gap: 8px` | Preset layout baris enum values / form item (pitch: 32 + 8 = 40 px). |
| `ROW_LAYOUT_CARD` | `height: 36px, padding_x: 11px, padding_y: 7px, handle: 16px, handle_gap: 12px, slot_gap: 8px, trailing: 20px, trailing_gap: 8px, row_gap: 5px` | Preset layout kartu daftar hirarki / panel kiri (pitch: 36 + 5 = 41 px). |

---

## 3. Aturan & Solusi Masalah Layout Egui

### A. Jebakan Padding Tombol Ikon (*Icon Button Padding Trap*)
* **Masalah**: Tema proyek (`theme::apply`) mengatur `button_padding = vec2(10.0, 4.0)`. Jika membuat tombol ikon dengan `ui.add_sized([28.0, 28.0], egui::Button::new(icon))`, egui akan menambahkan padding horizontal `2 * 10.0 = 20.0 px` ke lebar ikon (14 px), sehingga tombol melar menjadi **34x28 px** alih-alih 28x28 px. Hal ini menyebabkan baris toolbar melebar dan mendorong lebar panel utama.
* **Solusi**: Selalu bungkus tombol ikon bujur sangkar ke dalam `ui.scope` dengan `button_padding = Vec2::ZERO`, atau alokasikan dengan `allocate_exact_size` + `paint_icon`:
  ```rust
  let icon_btn = ui
      .scope(|ui| {
          ui.spacing_mut().button_padding = egui::Vec2::ZERO;
          ui.add_sized(
              [ICON_BUTTON_SIZE_COMPACT, ICON_BUTTON_SIZE_COMPACT],
              egui::Button::new(icon.rich(ui.ctx(), 14.0, t.ink_strong))
                  .fill(t.sunken)
                  .stroke(t.hairline())
                  .corner_radius(CornerRadius::same(3)),
          )
      })
      .inner;
  ```

---

### B. Gutter Scrollbar Mengurangi Lebar Baris (*Scroll Gutter Gaps*)
* **Masalah**: `ScrollStyle::thin()` memiliki default `floating_allocated_width = 6.0`. Saat daftar panjang dimasukkan ke dalam `ScrollArea::vertical()`, kartu di dalam daftar menjadi 6 px lebih sempit daripada input search di atasnya (muncul celah kanan 18 px alih-alih 12 px).
* **Solusi**: Setel `floating_allocated_width = 0.0` pada gaya scrollbar daftar tersebut:
  ```rust
  let mut scroll = egui::style::ScrollStyle::thin();
  scroll.floating_allocated_width = 0.0;
  ui.style_mut().spacing.scroll = scroll;
  ```
  *Catatan: Pastikan ikon paling kanan pada kartu (misal tombol kebab atau drag handle) berjarak minimal 10–11 px dari tepi kartu agar tidak tertutup indikator scrollbar tipis yang mengambang.*

---

### C. Kolom Label Form (*Form Label Column*)
* **Masalah**: Jika setiap kontrol diletakkan langsung setelah `ui.label(...)`, kontrol akan mulai pada posisi `x` yang berbeda-beda karena panjang teks label ("Enum Identifier" vs "Tileset:" vs "Tags:") tidak sama.
* **Solusi**: Alokasikan lebar tetap `FORM_LABEL_COL_W` (110.0 px) sebelum kontrol pertama:
  ```rust
  fn card_label(ui: &mut Ui, text: &str, t: &Tokens) {
      let (rect, _) = ui.allocate_exact_size(
          vec2(FORM_LABEL_COL_W, CONTROL_HEIGHT_FORM),
          Sense::hover(),
      );
      let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
      child.with_layout(Layout::left_to_right(Align::Center), |ui| {
          ui.label(RichText::new(text).strong().color(t.ink_strong));
      });
  }
  ```

---

### D. Keselarasan Sub-Tabs dan Toolbar Atas
* **Masalah**: Tombol aksi utama di panel kiri (misal "+ New Enum") memiliki tinggi 28 px dan mulai pada `y = WORKSPACE_PADDING (12 px)`. Jika sub-tab di panel kanan menggunakan padding default yang menghasilkan tinggi 22 px pada `y = 15 px`, header kedua panel tampak tidak sejajar.
* **Solusi**: Strip sub-tab wajib memiliki tinggi `CONTROL_HEIGHT_COMPACT (28 px)` dan mulai pada `y = WORKSPACE_PADDING (12 px)` agar tepat sejajar dengan panel samping.

---

## 4. Pengujian Geometri / Rect Test (*Pixel Locking*)

Setiap workspace baru wajib dilengkapi tes integrasi headless (misalnya `tests/<workspace>_workspace_rects_test.rs`) untuk mengunci tata letak:

1. **Setup Headless**:
   - Pasang font dan tema proyek:
     ```rust
     let ctx = Context::default();
     lc_canvas::theme::install_fonts(&ctx);
     lc_canvas::theme::apply(&ctx, lc_canvas::theme::ThemeChoice::Dark);
     ```
   - Render pada resolusi target standar (1366x768) dan kondisi scrolling (misal: 12 item pada tinggi 500 px).
2. **Koleksi Shape**:
   - Ekstrak seluruh `Shape::Rect` dan `Shape::Text` dari `FullOutput.shapes`.
   - Bersihkan tekstur setelah frame selesai: `output.textures_delta.clear()`.
3. **Daftar Assert Wajib (Toleransi 0.5 px)**:
   - Panel kiri terisi tepat pada `0.0 .. LEFT_PANEL_WIDTH`.
   - Seluruh elemen kolom kiri (tombol atas, search box, kartu baris) membentang tepat pada `12.0 .. (LEFT_PANEL_WIDTH - 12.0)`.
   - Tombol-tombol ikon bujur sangkar tepat 1:1 sesuai tiernya (28x28 atau 32x32).
   - Tepi kartu detail sama dengan batas area konten (`(LEFT_PANEL_WIDTH + P) .. (SCREEN_WIDTH - P)`).
   - Seluruh baris kontrol form mulai pada posisi horizontal (`x`) yang identik.
   - Tombol footer dan tombol toolbar kanan merapat ke tepi kanan konten (`SCREEN_WIDTH - P`).

---

## 5. List Rows and Cards (Standard Baris & Kartu Daftar)

Komponen daftar yang dapat diurutkan ulang (*reorderable lists*) dan kartu baris menggunakan komponen bersama `reorder_list` dan `reorder_row` dengan dua preset tata letak resmi dari `workspace_metrics`:

### Tabel Perbandingan Preset Layout (Dimensi Hasil Ukur A0)

| Parameter | `ROW_LAYOUT_VALUES` (Baris Nilai / Form) | `ROW_LAYOUT_CARD` (Kartu Daftar Panel Kiri) | Catatan Geometri A0 |
|---|---|---|---|
| **Tinggi (`height`)** | `32.0 px` (`CONTROL_HEIGHT_FORM`) | `36.0 px` | Sesuai tinggi kontrol form vs tinggi kartu daftar panel kiri |
| **Padding Horizontal (`padding_x`)** | `0.0 px` | `11.0 px` | 10 px padding isi kartu + 1 px garis tepi (*inside stroke*) |
| **Padding Vertikal (`padding_y`)** | `0.0 px` | `7.0 px` | 6 px padding isi kartu + 1 px garis tepi (*inside stroke*) |
| **Lebar Handle Drag (`handle_w`)** | `20.0 px` | `16.0 px` | Ikon drag handle berada di `x: 23..39` (pusat `x = 31.0`, glif `x = 24.0`) |
| **Jarak Setelah Handle (`handle_gap`)** | `8.0 px` | `12.0 px` | Teks nama mulai persis di `x = 51.0` (12 panel pad + 11 card pad + 16 handle + 12 gap) |
| **Jarak Antar Slot Body (`slot_gap`)** | `8.0 px` | `8.0 px` | Jarak seragam antar elemen/slot di dalam body baris |
| **Lebar Elemen Trailing (`trailing_w`)** | `32.0 px` | `20.0 px` | 32 px untuk tombol hapus (`Trailing::Delete`), 20 px untuk kebab menu (`Trailing::Kebab`) |
| **Jarak Sebelum Trailing (`trailing_gap`)** | `8.0 px` | `8.0 px` | Tombol kebab berada di `x: 217..237` (pusat `x = 227.0`, glif `x = 221.5`) |
| **Jarak Antar Baris (`row_gap`)** | `8.0 px` | `5.0 px` | Diterapkan oleh `reorder_list` setelah menyetel `item_spacing.y = 0` |
| **Pitch Vertikal Total** | **40.0 px** (32 + 8) | **41.0 px** (36 + 5) | Jarak vertikal absolut antar titik awal baris berurutan |
| **Jenis Trailing** | `Trailing::Delete` (ikon Trash 32x32) | `Trailing::Kebab` (ikon Kebab "⋮" 20x20) | Opsi `Trailing::None` juga didukung |

### Ketentuan Rendering Baris & Kartu
- **RowFrame (Bingkai Baris)**: the Card preset always draws its outline; the Values preset has no row frame by design (each widget draws its own).
  Pada preset Card, kartu digambar dengan sudut melengkung 4 px (`CornerRadius::same(4)`). Kartu yang tidak dipilih (*unselected*) memiliki garis tepi `t.hairline().color` (1.0 px) dan kartu yang sedang dipilih (*selected*) digambar dengan garis tepi aksen `t.accent` (1.0 px). Pada preset Values, tidak ada pengisian latar atau garis tepi tingkat baris; setiap widget/kontrol menggambar bingkainya sendiri. Tombol hapus (`Trailing::Delete`) selalu berbingkai (`icon_button` 32x32 dengan fill sunken, stroke 1 hairline, dan radius 3).
- **Interaktivitas & Aksesibilitas**:
  Kartu yang dapat diklik (`clickable: true`) menampilkan kursor tangan pointer (`CursorIcon::PointingHand`) saat di-hover. Komponen mendukung navigasi keyboard penuh (Arrow Up/Down untuk memilih item, Enter untuk klik/aktivasi, dan Alt + Arrow Up/Down untuk memindahkan urutan).
- **Larangan Menggambar Manual**:
  Pemanggil (*caller*) **TIDAK BOLEH** menggambar baris, garis tepi kartu, ikon drag handle, atau tombol kebab secara manual dengan `ui.painter()`. Semua dekorasi visual dan penanganan interaksi ditangani oleh `reorder_row` dan `reorder_list`.

---

## 6. Checklist Pembuatan Workspace Baru (New Workspace Checklist)

Setiap kali membuat workspace editor baru, ikuti checklist berikut:

> [!IMPORTANT]
> **A new workspace MUST use def_left_panel, reorder_list, and workspace_metrics; it MUST NOT define its own row drawing, card stroke, drag handle, or kebab code.**

- [ ] **Gunakan `def_left_panel`**: Panel daftar kiri wajib menggunakan fungsi bersama `def_left_panel` dari modul `shared::ui::def_workspace`.
- [ ] **Gunakan `reorder_list` & `reorder_row`**: Seluruh daftar yang memiliki urutan (baik kartu daftar panel kiri maupun baris nilai/properti di panel kanan) wajib dibangun di atas `reorder_list`.
- [ ] **Gunakan Token dari `workspace_metrics`**: Gunakan konstanta `ROW_LAYOUT_CARD`, `ROW_LAYOUT_VALUES`, `LEFT_PANEL_WIDTH`, `WORKSPACE_PADDING`, dan token standar lainnya. Dilarang menggunakan *magic numbers*.
- [ ] **Kunci Geometri dengan Rect Test**: Tambahkan tes integrasi headless (misalnya `tests/<name>_workspace_rects_test.rs`) yang memverifikasi tata letak pada resolusi 1366x768 dengan toleransi 0.5 px.
