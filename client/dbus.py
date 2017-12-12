from pydbus import SessionBus

bus = SessionBus()
notifications = bus.get('.Notifications')

notifications.Notify('test', 0, 'dialog-information', \
        "Hey!", "It works!", [], {}, 50000)
