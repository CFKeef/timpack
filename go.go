package cache

import (
	"context"
	"encoding/json"
	"errors"

	"github.com/nats-io/nats.go"
	"github.com/nats-io/nats.go/jetstream"
)

type Storage[V interface{}] interface {
	Get(ctx context.Context, key string) (*V, error)
	Put(ctx context.Context, key string, item V) error
	Delete(ctx context.Context, key string) error
}

type Memory[V any] struct {
	items map[string]V
}

func NewMemoryStorage[V any]() Storage[V] {
	return Memory[V]{
		items: make(map[string]V),
	}
}

func (m Memory[V]) Delete(ctx context.Context, key string) error {
	delete(m.items, key)

	return nil
}

func (m Memory[V]) Get(ctx context.Context, key string) (*V, error) {
	v, ok := m.items[key]

	if !ok {
		return nil, errors.New("missing item")
	}

	return &v, nil
}

func (m Memory[V]) Put(ctx context.Context, key string, item V) error {
	m.items[key] = item

	return nil
}

type Nats[V any] struct {
	name  string
	store jetstream.KeyValue
}

func NewNats[V any](name string, url string, cfg jetstream.KeyValueConfig) (Storage[V], error) {
	nc, err := nats.Connect(url)

	if err != nil {
		return nil, err
	}

	js, err := jetstream.New(nc)

	if err != nil {
		return nil, err
	}

	store, err := js.CreateKeyValue(context.Background(), cfg)

	if err != nil {
		return nil, err
	}

	return &Nats[V]{
		name:  name,
		store: store,
	}, nil
}

func (n *Nats[V]) Delete(ctx context.Context, key string) error {
	err := n.store.Delete(ctx, key)

	if err != nil {
		return err
	}

	return nil
}

func (n *Nats[V]) Get(ctx context.Context, key string) (*V, error) {
	raw, err := n.store.Get(ctx, key)

	if err != nil {
		return nil, err
	}

	var result V

	err = json.Unmarshal(raw.Value(), result)

	if err != nil {
		return nil, err
	}

	return &result, nil
}

func (n *Nats[V]) Put(ctx context.Context, key string, item V) error {
	b, err := json.Marshal(item)

	if err != nil {
		return err
	}

	err = n.put(ctx, key, b)

	if err != nil {
		return err
	}

	return nil
}

func (n *Nats[V]) put(ctx context.Context, key string, data []byte) error {
	_, err := n.store.Put(ctx, key, data)

	if err != nil {
		return err
	}

	return nil
}
