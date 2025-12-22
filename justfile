set dotenv-load
set positional-arguments

repo := `pwd`

BOLD := '\033[1m'
ITALIC := '\033[3m'
RESET := '\033[0m'
YELLOW2 := '\033[38;5;3m'
BLACK := '\033[30m'
RED := '\033[31m'
GREEN := '\033[32m'
YELLOW := '\033[33m'
BLUE := '\033[34m'
MAGENTA := '\033[35m'
CYAN := '\033[36m'
WHITE := '\033[37m'

default:
    @echo
    @echo "ta (Typescript Analyst)"
    @echo "------------------------------------"
    @echo "{{ITALIC}}Using {{BOLD}}OXC{{RESET}}{{ITALIC}} to interrogate and improve {{RESET}}"
    @echo "{{ITALIC}}Typescript code bases{{RESET}}"
    @echo "------------------------------------"
    @echo ""
    @just --list | grep -v 'default'
    @echo 


build *args="":
    @echo ""
    @echo "Build Rust and Typescript modules"
    @echo "---------------------------------"
    
    @echo ""
    @cargo build {{args}}
    @cd "{{repo}}/ts" && pnpm build

test *args="":
    @echo ""
    @echo "Testing Rust Modules"
    @echo "--------------------"
    @echo ""
    @cargo test {{args}}
