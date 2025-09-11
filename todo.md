Popraw w plikach manifestow, dokumentacji, kodu
- licencja apache 2
- autor: Tom Sapletta <info@softreck.dev>
- dodaj badges do README
- dodaj testy e2e
- zaktualizuj dokumentacje do aplikacji tauri

Dane wypelniana i logowania:
- aplikacja tauri powinna przed uruchomieniem systemu do wypelniania formualrzy posiadac juz sesje uzsera wraaz z danymi logowania, 
- stworz aplikacje tauri do zebrania zaleznosci koniecznych do uruchomienia autowypelniania formularzy czyli danych logowania do stron www w formie  bitwarde lub innego, ktory pwoinien byc uruchomiony w docker i zachowywac te dane nawet po ussunieciu samego docker images i containers,
- bitwarden lub inne programowanie do przechowywania hasel powinno byc podstawa wszelkich wymaganych danych prodczas uzupelniania formularzy
- przegaldarka uzywana przez tauri powinna bmiec wgrany plugin do obslugi bitwarden lub znajdz inne rozwiazanie