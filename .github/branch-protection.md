# Настройка защиты веток GitHub

## Обязательные настройки для main/master ветки

Для обеспечения качества кода и обязательного прохождения CI, необходимо настроить защиту веток в GitHub:

### 1. Перейдите в настройки репозитория
- Repository Settings → Branches → Add rule

### 2. Настройте правило для main/master ветки

**Branch name pattern:** `main` (или `master`)

**Защита ветки:**
- ✅ **Require a pull request before merging**
  - ✅ Require approvals: `1` (минимум)
  - ✅ Dismiss stale PR approvals when new commits are pushed
  - ✅ Require review from code owners

- ✅ **Require status checks to pass before merging**
  - ✅ Require branches to be up to date before merging
  - ✅ Status checks that are required:
    - `test` (Test job)
    - `coverage` (Coverage (Blocking) job)

- ✅ **Require conversation resolution before merging**

- ✅ **Require signed commits**

- ✅ **Require linear history**

- ✅ **Include administrators**

### 3. Дополнительные настройки

**Code owners:**
Создайте файл `.github/CODEOWNERS`:
```
# Эти люди будут автоматически запрашиваться для ревью
* @maintainer1 @maintainer2
```

### 4. Настройки для feature веток

Создайте дополнительное правило для feature веток:
**Branch name pattern:** `feature/*`

**Защита ветки:**
- ✅ **Require status checks to pass before merging**
  - ✅ Require branches to be up to date before merging
  - ✅ Status checks that are required:
    - `test`
    - `coverage`

### 5. Проверка настроек

После настройки:
1. Попробуйте создать PR без прохождения CI
2. Убедитесь, что кнопка "Merge" заблокирована
3. Проверьте, что CI проверки обязательны

---

**Важно:** Эти настройки гарантируют, что никакой код не попадет в main ветку без прохождения всех проверок качества! 