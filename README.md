# Sentinel

Sentinel is a powerful and flexible framework for monitoring and controlling distributed systems. It provides a secure lab monitoring daemon.

## Plan:

So for the initial plan would be start with a proof-of-concept for our architecture in python, then slowly port to rust.

## Installation:

### Step 1:

#### Install the sentinel in root computer.

```bash
git clone https://github.com/Balamurugan1962/Sentinel.git
```

### Step 2:

#### Install the dependancy.

```bash
cd poc-python
pip install -r requirements.txt
```

### Step 4:

#### Configure the system.

In the root project open `config.toml` and the ip for both root and node computers.

### Step 3:

#### Run the main.py with root access.

```bash
python run main.py
```

main.py will do the initial set-up by

- going through the config file
- setting up clients in node computers
- checking if everything works properly

after that our agent will start runing, capturing the events from node computers
